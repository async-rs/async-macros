#![allow(non_snake_case)]

/// Awaits multiple futures simultaneously, returning all results once complete.
///
/// While `join!(a, b)` is similar to `(a.await, b.await)`,
/// `join!` polls both futures concurrently and therefore is more efficent.
///
/// This macro is only usable inside of async functions, closures, and blocks.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use async_macros::join;
/// use futures::future;
///
/// let a = future::ready(1u8);
/// let b = future::ready(2u8);
///
/// assert_eq!(join!(a, b).await, (1, 2));
/// # });
/// ```
#[macro_export]
macro_rules! join {
    ($($fut:ident),* $(,)?) => { {
        async {
            $(
                // Move future into a local so that it is pinned in one place and
                // is no longer accessible by the end user.
                let mut $fut = $crate::MaybeDone::new($fut);
            )*
            $crate::utils::poll_fn(move |cx| {
                use $crate::utils::future::Future;
                use $crate::utils::task::Poll;
                use $crate::utils::pin::Pin;

                let mut all_done = true;
                $(
                    let fut = unsafe { Pin::new_unchecked(&mut $fut) };
                    all_done &= Future::poll(fut, cx).is_ready();
                )*
                if all_done {
                    Poll::Ready(($(
                        unsafe { Pin::new_unchecked(&mut $fut) }.take().unwrap(),
                    )*))
                } else {
                    Poll::Pending
                }
            }).await
        }
    } }
}
