use thiserror::Error;

use std::fmt::Display;

/// Guards an invariant, returning an error if violated. Like `anyhow::ensure!`.
///
/// When passed a format string, wraps the message in [`Assumption`] and converts
/// it into the function's error type via `Into`. When passed an error value
/// directly, converts it via `Into` unchanged.
///
/// # Examples
///
/// ```rust,ignore
/// assume!(panels.len() > 0, "panels should never be empty");
/// assume!(!value.is_empty(), "expected non-empty value, got: {value}");
/// assume!(!value.is_empty(), "expected non-empty value, got: {}", value);
/// assume!(!value.is_empty(), Error::Empty);
/// ```
macro_rules! assume {
    ($cond:expr, $fmt:literal $(, $($arg:tt)*)?) => {
        if $crate::stdx::hint::unlikely(!$cond) {
            return Err($crate::stdx::error::Assumption::from(
                format!("`{}`: {}", stringify!($cond), format!($fmt $(, $($arg)*)?))
            ).into());
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if $crate::stdx::hint::unlikely(!$cond) {
            return Err($err.into());
        }
    };
}

/// Guards a pattern match, returning an error if the value does not match, like `assert_matches!`.
///
/// Binds variables from the pattern into the surrounding scope. Supports an optional
/// match guard via `if`. When passed a format string, wraps the message in [`Assumption`]
/// and converts it into the function's error type via `Into`. When passed an error value
/// directly, converts it via `Into` unchanged.
///
/// # Examples
///
/// ```rust,ignore
/// // Pattern only - no binding needed
/// assume_matches!(value, Some(_), "expected Some");
///
/// // With binding
/// assume_matches!(value, Some(inner), "expected Some");
/// // `inner` is now in scope
///
/// // With guard
/// assume_matches!(published, Some(pub) if pub.year() >= 2014, "published year must be at least 2014");
///
/// // With arbitrary error
/// assume_matches!(value, Some(_), ParseError::Missing);
/// ```
macro_rules! assume_matches {
    // No guard, literal only
    ($expr:expr, $pat:pat, $fmt:literal $(,)?) => {
        let $pat = $expr else {
            return Err($crate::stdx::error::Assumption::from(
                format!($fmt)
            ).into());
        };
    };
    // No guard, format args
    ($expr:expr, $pat:pat, $fmt:literal, $($arg:tt)+) => {
        let $pat = $expr else {
            return Err($crate::stdx::error::Assumption::from(
                format!($fmt, $($arg)+)
            ).into());
        };
    };
    // No guard, arbitrary error
    ($expr:expr, $pat:pat, $err:expr $(,)?) => {
        let $pat = $expr else {
            return Err($err.into());
        };
    };
    // With guard, literal only
    ($expr:expr, $pat:pat if $guard:expr, $fmt:literal $(,)?) => {
        let $pat = $expr else {
            return Err($crate::stdx::error::Assumption::from(
                format!($fmt)
            ).into());
        };
        if $crate::stdx::hint::unlikely(!$guard) {
            return Err($crate::stdx::error::Assumption::from(
                format!($fmt)
            ).into());
        }
    };
    // With guard, format args
    ($expr:expr, $pat:pat if $guard:expr, $fmt:literal, $($arg:tt)+) => {
        let $pat = $expr else {
            return Err($crate::stdx::error::Assumption::from(
                format!($fmt, $($arg)+)
            ).into());
        };
        if $crate::stdx::hint::unlikely(!$guard) {
            return Err($crate::stdx::error::Assumption::from(
                format!($fmt, $($arg)+)
            ).into());
        }
    };
    // With guard, arbitrary error
    ($expr:expr, $pat:pat if $guard:expr, $err:expr $(,)?) => {
        let $pat = $expr else {
            return Err($err.into());
        };
        if $crate::stdx::hint::unlikely(!$guard) {
            return Err($err.into());
        }
    };
}

/// Unconditionally returns an error. Like `anyhow::bail!`.
///
/// When passed a format string, wraps the message in [`Assumption`] and converts
/// it into the function's error type via `Into`. When passed an error value
/// directly, converts it via `Into` unchanged.
///
/// # Examples
///
/// ```rust,ignore
/// assumption!("something went wrong");
/// assumption!("something went wrong with {value}");
/// assumption!("something went wrong with {}", value);
/// assumption!(PostIdParseError::MissingWebtoonType);
/// ```
macro_rules! assumption {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        return Err($crate::stdx::error::Assumption::from(
            format!($fmt $(, $($arg)*)?)
        ).into())
    };
    ($err:expr $(,)?) => {
        return Err($err.into())
    };
}

pub(crate) use assume;
pub(crate) use assume_matches;
pub(crate) use assumption;

/// A violated internal invariant - always indicates a bug in the library.
///
/// If you encounter this error, please open an issue. It is not actionable from
/// user code and can only be resolved by an internal fix.
///
/// `Assumption` is used exclusively when interacting with the platform, not for
/// validating user input. Liberal use of pre- and post-condition checks ensures
/// that unexpected platform changes are caught early.
#[derive(Debug, Error)]
#[error("internal assumption violated: {0}")]
pub struct Assumption(String);

impl From<String> for Assumption {
    #[inline]
    fn from(msg: String) -> Self {
        Self(msg)
    }
}

/// Attaches an [`Assumption`] context message to an `Option` or `Result`.
///
/// Analogous to `.context()` in `anyhow` - use `.assumption()` for a static
/// message and `.with_assumption()` when the message needs to be constructed
/// lazily.
pub trait Assume<T> {
    type Output;

    /// Converts `self` into a `Result`, using `msg` as the [`Assumption`] context.
    ///
    /// For `Result`, the original error is appended: `"{msg}: {err}"`.
    fn assumption(self, msg: &'static str) -> Self::Output;

    /// Like [`assumption`](Assume::assumption), but constructs the message lazily.
    fn with_assumption(self, msg: impl FnOnce() -> String) -> Self::Output;
}

impl<T> Assume<T> for Option<T> {
    type Output = Result<T, Assumption>;

    #[inline]
    fn assumption(self, msg: &'static str) -> Self::Output {
        self.ok_or_else(|| Assumption(msg.into()))
    }

    #[inline]
    fn with_assumption(self, msg: impl FnOnce() -> String) -> Self::Output {
        self.ok_or_else(|| Assumption(msg()))
    }
}

impl<T, E: Display> Assume<T> for Result<T, E> {
    type Output = Result<T, Assumption>;

    #[inline]
    fn assumption(self, msg: &'static str) -> Self::Output {
        self.map_err(|err| Assumption(format!("{msg}: {err}")))
    }

    #[inline]
    fn with_assumption(self, msg: impl FnOnce() -> String) -> Self::Output {
        self.map_err(|err| Assumption(format!("{}: {err}", msg())))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::assert_matches;

    // --- assumption! ---

    #[test]
    fn assumption_with_literal() {
        let result: Result<(), Assumption> = (|| {
            assumption!("failed to uphold assumption");
        })();
        assert_matches!(
            result,
            Err(Assumption(msg)) if msg == "failed to uphold assumption"
        );
    }

    #[test]
    fn assumption_with_inline_format_arg() {
        let arg = "foo";
        let result: Result<(), Assumption> = (|| {
            assumption!("failed to uphold assumption with {arg}");
        })();
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "internal assumption violated: failed to uphold assumption with foo"
        );
    }

    #[test]
    fn assumption_with_format_args() {
        let arg = "foo";
        let result: Result<(), Assumption> = (|| {
            assumption!("failed to uphold assumption with {arg} and {}", "bar");
        })();
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "internal assumption violated: failed to uphold assumption with foo and bar"
        );
    }

    #[test]
    fn assumption_with_arbitrary_error() {
        let result: Result<(), std::io::Error> = (|| {
            assumption!(std::io::Error::other("something went wrong"));
        })();
        assert!(result.is_err());
    }

    // --- assume! ---

    #[test]
    fn assume_does_not_error_when_condition_holds() {
        let panels = [()];
        let result: Result<(), Assumption> = (|| {
            assume!(!panels.is_empty(), "episode panels should not be empty");
            Ok(())
        })();
        assert!(result.is_ok());
    }

    #[test]
    fn assume_with_literal() {
        let panels: Vec<()> = vec![];
        let result: Result<(), Assumption> = (|| {
            assume!(!panels.is_empty(), "episode panels should not be empty");
            Ok(())
        })();
        assert!(matches!(result, Err(Assumption(_))));
    }

    #[test]
    fn assume_with_inline_format_arg() {
        let panels: Vec<()> = vec![];
        let result: Result<(), Assumption> = (|| {
            assume!(
                !panels.is_empty(),
                "episode panels should not be empty, got: {panels:?}"
            );
            Ok(())
        })();
        let err = result.unwrap_err();
        assert!(
            err.to_string()
                .contains("episode panels should not be empty, got: []")
        );
    }

    #[test]
    fn assume_with_format_args() {
        let panels: Vec<()> = vec![];
        let result: Result<(), Assumption> = (|| {
            assume!(
                !panels.is_empty(),
                "episode panels should not be empty, got: {}",
                panels.len()
            );
            Ok(())
        })();
        let err = result.unwrap_err();
        assert!(
            err.to_string()
                .contains("episode panels should not be empty, got: 0")
        );
    }

    #[test]
    fn assume_with_arbitrary_error() {
        let panels: Vec<()> = vec![];
        let result: Result<(), std::io::Error> = (|| {
            assume!(!panels.is_empty(), std::io::Error::other("no panels found"));
            Ok(())
        })();
        assert!(result.is_err());
    }

    // --- assume_matches! ---

    #[test]
    fn assume_matches_succeeds_on_matching_pattern() {
        let result: Result<(), Assumption> = (|| {
            let value: Option<u32> = Some(1);
            assume_matches!(value, Some(_), "expected Some");
            Ok(())
        })();
        assert!(result.is_ok());
    }

    #[test]
    fn assume_matches_errors_on_non_matching_pattern() {
        let result: Result<(), Assumption> = (|| {
            let value: Option<u32> = None;
            assume_matches!(value, Some(_), "expected Some");
            Ok(())
        })();
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "internal assumption violated: expected Some"
        );
    }

    #[test]
    fn assume_matches_binds_inner_value() {
        let result: Result<(), Assumption> = (|| {
            let value: Option<u32> = Some(42);
            assume_matches!(value, Some(inner), "expected Some");
            assert_eq!(inner, 42);
            Ok(())
        })();
        assert!(result.is_ok());
    }

    #[test]
    fn assume_matches_succeeds_when_guard_holds() {
        let result: Result<(), Assumption> = (|| {
            let published: Option<i32> = Some(2024);
            assume_matches!(published, Some(year) if year >= 2014, "published year must be at least 2014");
            Ok(())
        })();
        assert!(result.is_ok());
    }

    #[test]
    fn assume_matches_errors_when_guard_fails() {
        let result: Result<(), Assumption> = (|| {
            let published: Option<i32> = Some(2010);
            assume_matches!(published, Some(year) if year >= 2014, "published year must be at least 2014");
            Ok(())
        })();
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "internal assumption violated: published year must be at least 2014"
        );
    }

    #[test]
    fn assume_matches_errors_on_none_with_guard() {
        let result: Result<(), Assumption> = (|| {
            let published: Option<i32> = None;
            assume_matches!(published, Some(year) if year >= 2014, "published year must be at least 2014");
            Ok(())
        })();
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "internal assumption violated: published year must be at least 2014"
        );
    }

    #[test]
    fn assume_matches_with_arbitrary_error() {
        let result: Result<(), std::io::Error> = (|| {
            let value: Option<u32> = None;
            assume_matches!(value, Some(_), std::io::Error::other("expected Some"));
            Ok(())
        })();
        assert!(result.is_err());
    }

    #[test]
    fn assume_matches_with_format_args() {
        let result: Result<(), Assumption> = (|| {
            let value: Option<u32> = None;
            assume_matches!(value, Some(_), "expected Some, got: {}", "None");
            Ok(())
        })();
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "internal assumption violated: expected Some, got: None"
        );
    }

    // --- .assumption() / .with_assumption() ---

    #[test]
    fn option_assumption() {
        let val: Option<()> = None;
        let result: Result<(), Assumption> = val.assumption("failed to find tag");
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "internal assumption violated: failed to find tag"
        );
    }

    #[test]
    fn option_with_assumption() {
        let webtoon = "Test";
        let val: Option<()> = None;
        let result: Result<(), Assumption> =
            val.with_assumption(|| format!("failed to find tag for `{webtoon}`"));
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "internal assumption violated: failed to find tag for `Test`"
        );
    }

    #[test]
    fn option_assumption_is_ok_when_some() {
        let val: Option<()> = Some(());
        let result: Result<(), Assumption> = val.assumption("should not trigger");
        assert!(result.is_ok());
    }

    #[test]
    fn result_assumption_includes_original_error() {
        let val: Result<(), &str> = Err("original error");
        let result: Result<(), Assumption> = val.assumption("context message");
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "internal assumption violated: context message: original error"
        );
    }

    #[test]
    fn result_with_assumption_includes_original_error() {
        let webtoon = "Test";
        let val: Result<(), &str> = Err("original error");
        let result: Result<(), Assumption> =
            val.with_assumption(|| format!("context message for `{webtoon}`"));
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "internal assumption violated: context message for `Test`: original error"
        );
    }

    #[test]
    fn result_assumption_is_ok_when_ok() {
        let val: Result<(), &str> = Ok(());
        let result: Result<(), Assumption> = val.assumption("should not trigger");
        assert!(result.is_ok());
    }
}
