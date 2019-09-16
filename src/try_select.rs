#![allow(non_snake_case)]

/// Waits for either one of several similarly-typed futures to complete.
///
/// Awaits multiple futures simultaneously, returning all results once complete.
///
/// `try_select!` is similar to [`select!`], but keeps going if a future
/// resolved to an error until all futures have been resolved. In which case
/// the error of the last item in the list will be returned.
///
/// This macro is only usable inside of async functions, closures, and blocks.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// # async fn main() -> Result<(), std::io::Error> {
/// use async_macros::try_select;
/// use futures::future;
/// use std::io::{Error, ErrorKind};
///
/// let a = future::pending::<Result<_, Error>>();
/// let b = future::ready(Err(Error::from(ErrorKind::Other)));
/// let c = future::ready(Ok(1u8));
///
/// assert_eq!(try_select!(a, b, c).await?, 1u8);
/// # Ok(())
/// # }
/// # main().await.unwrap();
/// # });
/// ```
#[macro_export]
macro_rules! try_select {
    ($($fut:ident),+ $(,)?) => { {
        async {
            use $crate::utils::future::Future;
            use $crate::utils::pin::Pin;
            use $crate::utils::poll_fn;
            use $crate::utils::result::Result;
            use $crate::utils::task::Poll;

            $(
                // Move future into a local so that it is pinned in one place and
                // is no longer accessible by the end user.
                let mut $fut = $crate::MaybeDone::new($fut);
            )*

            let res: Result<_, _> = poll_fn(move |cx| {
                let mut all_done = true;

                $(
                    let fut = unsafe { Pin::new_unchecked(&mut $fut) };
                    if Future::poll(fut, cx).is_ready() {
                        let fut = Pin::new(&$fut);
                        if fut.as_ref().unwrap().is_ok() {
                            let fut = unsafe { Pin::new_unchecked(&mut $fut) };
                            let res = fut.take().unwrap();
                            return Poll::Ready(res);
                        } else {
                            all_done = false;
                        }
                    } else {
                        all_done = false;
                    }
                )*

                if all_done {
                    // We need to iterate over all items to get the last error.
                    let mut err = None;
                    $(
                        if err.is_none() {
                            let fut = unsafe { Pin::new_unchecked(&mut $fut) };
                            err = Some(fut.take().unwrap());
                        }
                    )*
                    return Poll::Ready(err.unwrap());
                } else {
                    Poll::Pending
                }
            }).await;
            res
        }
    } }
}
