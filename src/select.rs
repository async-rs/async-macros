#![allow(non_snake_case)]

/// Waits for either one of several similarly-typed futures to complete.
///
/// Awaits multiple futures simultaneously, returning all results once complete.
///
/// This function will return a new future which awaits for either one of both
/// futures to complete. If multiple futures are completed at the same time,
/// resolution will occur in the order that they have been passed.
///
/// Note that this macro consumes all futures passed, and once a future is
/// completed, all other futures are dropped.
///
/// This macro is only usable inside of async functions, closures, and blocks.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use async_macros::select;
/// use futures::future;
///
/// let a = future::pending();
/// let b = future::ready(1u8);
/// let c = future::ready(2u8);
///
/// assert_eq!(select!(a, b).await, 1u8);
/// # });
/// ```
#[macro_export]
macro_rules! select {
    ($($fut:ident),* $(,)?) => { {
        async {
            $(
                // Move future into a local so that it is pinned in one place and
                // is no longer accessible by the end user.
                let mut $fut = $crate::maybe_done($fut);
            )*
            $crate::utils::poll_fn(move |cx| {
                use $crate::utils::future::Future;
                use $crate::utils::task::Poll;
                use $crate::utils::pin::Pin;

                $(
                    let fut = unsafe { Pin::new_unchecked(&mut $fut) };
                    if Future::poll(fut, cx).is_ready() {
                        let fut = unsafe { Pin::new_unchecked(&mut $fut) };
                        let output = fut.take_output().unwrap();
                        return Poll::Ready(output);
                    }
                )*

                // If nothing matched we return Pending.
                Poll::Pending
            }).await
        }
    } }
}
