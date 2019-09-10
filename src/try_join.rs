/// Polls multiple futures simultaneously, resolving to a [`Result`] containing
/// either a tuple of the successful outputs or an error.
///
/// `try_join!` is similar to [`join!`], but completes immediately if any of
/// the futures return an error.
///
/// This macro is only usable inside of async functions, closures, and blocks.
/// It is also gated behind the `async-await` feature of this library, which is
/// _not_ activated by default.
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
            $(
                // Move future into a local so that it is pinned in one place and
                // is no longer accessible by the end user.
                let mut $fut = $crate::future::maybe_done($fut);
            )*

            let res: $crate::utils::result::Result<_, _> = $crate::utils::poll_fn(move |cx| {
                let mut all_done = true;
                $(
                    if $crate::utils::future::Future::poll(
                        unsafe { $crate::utils::pin::Pin::new_unchecked(&mut $fut) }, cx).is_pending()
                    {
                        all_done = false;
                    } else if unsafe { $crate::utils::pin::Pin::new_unchecked(&mut $fut) }.output_mut().unwrap().is_err() {
                        // `.err().unwrap()` rather than `.unwrap_err()` so that we don't introduce
                        // a `T: Debug` bound.
                        return $crate::utils::task::Poll::Ready(
                            $crate::utils::result::Result::Err(
                                unsafe { $crate::utils::pin::Pin::new_unchecked(&mut $fut) }.take_output().unwrap().err().unwrap()
                            )
                        );
                    }
                )*
                if all_done {
                    $crate::utils::task::Poll::Ready(
                        $crate::utils::result::Result::Ok(($(
                            // `.ok().unwrap()` rather than `.unwrap()` so that we don't introduce
                            // an `E: Debug` bound.
                            unsafe { $crate::utils::pin::Pin::new_unchecked(&mut $fut) }.take_output().unwrap().ok().unwrap(),
                        )*))
                    )
                } else {
                    $crate::utils::task::Poll::Pending
                }
            }).await;
            res
        }
    } }
}
