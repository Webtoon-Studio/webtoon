use thiserror::Error;

macro_rules! assumption {
    ($msg:literal $(, $args:expr)* ) => {{
        return Err($crate::stdx::error::Assumption::from( format!($msg $(, $args)*)).into());
    }};
    ($err:expr) => {{
        return Err($crate::stdx::error::Assumption::from( format!("{}", $err)).into());
    }};
    ($cond:expr, $msg:literal $(, $args:expr)* ) => {{
        if !$cond {
        return Err($crate::stdx::error::Assumption::from( format!("`{}`, {}", stringify!($cond), format!($msg $(, $args)*))).into());
        }
    }};
}

pub(crate) use assumption;

/// Represents internal assumptions that were violated.
///
/// If this is returned, this is considered a bug that must be fixed!
///
/// This error is not actionable by library user, and must be fixed via internal
/// code changes! Please open an issue!
///
/// # Use
///
/// The rule of thumb for this error is that it is only used when interacting
/// with the platform, as opposed to input data that might be passed to the library.
///
/// In an effort to maintain correctness in the library, liberal use of pre and
/// post checks are used to make sure any changes that happen underneath the
/// library are caught and fixed as soon as possible.
#[derive(Debug, Error)]
#[error("internal assumption violated: {0}")]
pub struct Assumption(String);

impl From<String> for Assumption {
    #[inline]
    fn from(msg: String) -> Self {
        Self(msg)
    }
}

pub trait Assume<T> {
    type Output;

    fn assumption(self, msg: &'static str) -> Self::Output;
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

impl<T, E> Assume<T> for Result<T, E> {
    type Output = Result<T, Assumption>;

    #[inline]
    fn assumption(self, msg: &'static str) -> Self::Output {
        self.map_err(|_err: _| Assumption(msg.into()))
    }

    #[inline]
    fn with_assumption(self, msg: impl FnOnce() -> String) -> Self::Output {
        self.map_err(|_err: _| Assumption(msg()))
    }
}

pub trait AssumeFor<T, E>: Assume<T> {
    fn assumption_for(self, msg: impl FnOnce(E) -> String) -> Self::Output;
}

impl<T, E> AssumeFor<T, E> for Result<T, E> {
    #[inline]
    fn assumption_for(self, msg: impl FnOnce(E) -> String) -> Self::Output {
        self.map_err(|err| Assumption(msg(err)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_bail_with_message() -> Result<(), Assumption> {
        assumption!("failed to uphold assumption");
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_bail_on_condition_fail_with_message() -> Result<(), Assumption> {
        let panels: Vec<()> = vec![];
        assumption!(
            !panels.is_empty(),
            "episode panels should not be empty, but was {}",
            panels.len()
        );
        Ok(())
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_have_format_args() -> Result<(), Assumption> {
        let arg = "foo";
        assumption!("failed to uphold assumption with {arg} and {}", "bar");
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_error_with_internal_assumption() -> Result<(), Assumption> {
        let err: Option<()> = None;
        err.assumption("failed to find `a.img` html tag on webtoon homepage")?;
        Ok(())
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_error_with_internal_assumption2() -> Result<(), Assumption> {
        let webtoon = "Test";
        let err: Option<()> = None;
        err.with_assumption(|| {
            format!("failed to find `a.img` html tag on webtoon homepage for `{webtoon}`")
        })?;
        Ok(())
    }
}
