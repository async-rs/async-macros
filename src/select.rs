#![allow(non_snake_case)]

/// Awaits multiple futures simultaneously, returning all results once complete.
///
/// While `join!(a, b)` is similar to `(a.await, b.await)`,
/// `join!` polls both futures concurrently and therefore is more efficent.
///
/// This macro is only usable inside of async functions, closures, and blocks.
/// It is also gated behind the `async-await` feature of this library, which is
/// _not_ activated by default.
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
