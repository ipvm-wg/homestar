//! Macros for cross-crate export.

/// Return early with an error.
///
/// Modelled after [anyhow::bail].
///
/// # Example
///
/// ```
/// use homestar_invocation::{bail, Error, Unit};
///
/// fn has_permission(user: usize, resource: usize) -> bool {
///      true
/// }
///
/// # fn main() -> Result<(), Error<Unit>> {
/// #     let user = 0;
/// #     let resource = 0;
/// #
///
/// if !has_permission(user, resource) {
///     bail!(Error::Unknown);
/// }
///
/// #    Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        return Err($e);
    };
}

/// Return early with an error if a condition is not satisfied.
///
/// Analogously to `assert!`, `ensure!` takes a condition and exits the function
/// if the condition fails. Unlike `assert!`, `ensure!` returns an `Error`
/// rather than panicking.
///
/// Modelled after [anyhow::ensure].
///
/// # Example
///
/// ```
/// use homestar_invocation::{ensure, Error, Unit};
///
/// #
/// # fn main() -> Result<(), Error<Unit>> {
/// #     let user = 1;
/// #
/// ensure!(
///     user < 2,
///     Error::ConditionNotMet(
///         "only user 0 and 1 are allowed".to_string()
///     )
/// );
/// #     Ok(())
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            bail!($e);
        }
    };
}
