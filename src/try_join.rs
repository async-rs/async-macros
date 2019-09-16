/// Awaits multiple fallible futures simultaneously, returning all results once
/// complete.
///
/// `try_join!` is similar to [`join!`], but completes immediately if any of
/// the futures return an error.
///
/// This macro is only usable inside of async functions, closures, and blocks.
///
/// # Examples
///
/// When used on multiple futures that return `Ok`, `try_join!` will return
/// `Ok` of a tuple of the values:
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use async_macros::try_join;
/// use futures::future;
///
/// let a = future::ready(Ok::<i32, i32>(1));
/// let b = future::ready(Ok::<u64, i32>(2));
///
/// assert_eq!(try_join!(a, b).await, Ok((1, 2)));
/// # });
/// ```
///
/// If one of the futures resolves to an error, `try_join!` will return
/// that error:
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use async_macros::try_join;
/// use futures::future;
///
/// let a = future::ready(Ok::<i32, i32>(1));
/// let b = future::ready(Err::<u64, i32>(2));
///
/// assert_eq!(try_join!(a, b).await, Err(2));
/// # });
/// ```
#[macro_export]
macro_rules! try_join {
    ($($fut:ident),* $(,)?) => { {
        async {
            use $crate::utils::future::Future;
            use $crate::utils::pin::Pin;
            use $crate::utils::poll_fn;
            use $crate::utils::result::Result;
            use $crate::utils::task::Poll;

            $(
                // Move future into a local so that it is pinned in one place and
                // is no longer accessible by the end user.
                let mut $fut = $crate::maybe_done($fut);
            )*

            let res: Result<_, _> = poll_fn(move |cx| {
                let mut all_done = true;
                $(
                    let fut = unsafe { Pin::new_unchecked(&mut $fut) };
                    if Future::poll(fut, cx).is_pending() {
                        all_done = false;
                    } else if unsafe { Pin::new_unchecked(&mut $fut) }.as_mut().unwrap().is_err() {
                        // `.err().unwrap()` rather than `.unwrap_err()` so that we don't introduce
                        // a `T: Debug` bound.
                        return Poll::Ready(
                            Result::Err(unsafe { Pin::new_unchecked(&mut $fut) }
                                .take()
                                .unwrap()
                                .err()
                                .unwrap()
                        ));
                    }
                )*
                if all_done {
                    let res = ($(
                        // `.ok().unwrap()` rather than `.unwrap()` so that we don't introduce
                        // an `E: Debug` bound.
                        unsafe { Pin::new_unchecked(&mut $fut) }
                            .take()
                            .unwrap()
                            .ok()
                            .unwrap(),
                    )*);
                    Poll::Ready(Result::Ok(res))
                } else {
                    Poll::Pending
                }
            }).await;
            res
        }
    } }
}
