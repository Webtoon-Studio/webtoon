use thiserror::Error;

macro_rules! invariant {
    ($msg:literal $(, $args:expr)* ) => {{
        return Err($crate::stdx::error::InternalInvariant::from( format!($msg $(, $args)*)).into());
    }};
    ($err:expr) => {{
        return Err($crate::stdx::error::InternalInvariant::from( format!("{}", $err)).into());
    }};
    ($cond:expr, $msg:literal $(, $args:expr)* ) => {{
        if !$cond {
        return Err($crate::stdx::error::InternalInvariant::from( format!("`{}`, {}", stringify!($cond), format!($msg $(, $args)*))).into());
        }
    }};
}

pub(crate) use invariant;

/// Represents internal invariants that were violated.
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
#[error("internal invariant violated: {0}")]
pub struct InternalInvariant(String);

impl From<String> for InternalInvariant {
    #[inline]
    fn from(msg: String) -> Self {
        Self(msg)
    }
}

pub trait Invariant<T> {
    type Output;

    fn invariant(self, msg: impl Into<String>) -> Self::Output;
}

impl<T> Invariant<T> for Option<T> {
    type Output = Result<T, InternalInvariant>;

    #[inline]
    fn invariant(self, msg: impl Into<String>) -> Self::Output {
        self.ok_or_else(|| InternalInvariant(msg.into()))
    }
}

impl<T, E> Invariant<T> for Result<T, E> {
    type Output = Result<T, InternalInvariant>;

    #[inline]
    fn invariant(self, msg: impl Into<String>) -> Self::Output {
        self.map_err(|_err: _| InternalInvariant(msg.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_bail_with_message() -> Result<(), InternalInvariant> {
        invariant!("failed to uphold assumption");
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_bail_on_condition_fail_with_message() -> Result<(), InternalInvariant> {
        let panels: Vec<()> = vec![];
        invariant!(
            !panels.is_empty(),
            "episode panels should not be empty, but was {}",
            panels.len()
        );
        Ok(())
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_have_format_args() -> Result<(), InternalInvariant> {
        let arg = "foo";
        invariant!("failed to uphold assumption with {arg} and {}", "bar");
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_error_with_internal_invariant() -> Result<(), InternalInvariant> {
        let err: Option<()> = None;
        err.invariant("failed to find `a.img` html tag on webtoon homepage")?;
        Ok(())
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_error_with_internal_invariant2() -> Result<(), InternalInvariant> {
        let webtoon = "Test";
        let err: Option<()> = None;
        err.invariant(format!(
            "failed to find `a.img` html tag on webtoon homepage for `{webtoon}`"
        ))?;
        Ok(())
    }
}
