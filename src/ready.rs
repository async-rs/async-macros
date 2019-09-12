/// Extracts the successful type of a `Poll<T>`.
///
/// This macro bakes in propagation of `Pending` signals by returning early.
#[macro_export]
macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            $crate::utils::task::Poll::Ready(t) => t,
            $crate::utils::task::Poll::Pending => return $crate::utils::task::Poll::Pending,
        }
    };
}
