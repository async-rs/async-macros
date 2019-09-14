#![allow(non_snake_case)]

/// Waits for either one of several similarly-typed futures to complete.
/// Awaits multiple futures simultaneously, returning all results once complete.
///
/// `try_select!` is similar to [`select!`], but keeps going if a future
/// resolved to an error until all futures have been resolved. In which case
/// the last error found will be returned.
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
/// let a = future::pending::<Result<u8, Error>>();
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
            $(
                // Move future into a local so that it is pinned in one place and
                // is no longer accessible by the end user.
                let mut $fut = $crate::maybe_done($fut);
            )*
            $crate::utils::poll_fn(move |cx| {
                use $crate::utils::future::Future;
                use $crate::utils::task::Poll;
                use $crate::utils::pin::Pin;

                let mut all_done = true;

                $(
                    let fut = unsafe { Pin::new_unchecked(&mut $fut) };
                    if Future::poll(fut, cx).is_ready() {
                        let fut = Pin::new(&$fut);
                        if fut.output().unwrap().is_ok() {
                            let fut = unsafe { Pin::new_unchecked(&mut $fut) };
                            let output = fut.take_output().unwrap();
                            return Poll::Ready(output);
                        } else {
                            all_done = false;
                        }
                    } else {
                        all_done = false;
                    }
                )*

                if all_done {
                    // We need to iterate over all items to not get an
                    // "unreachable code" warning.
                    let mut err = None;
                    $(
                        if err.is_none() {
                            let fut = unsafe { Pin::new_unchecked(&mut $fut) };
                            err = Some(fut.take_output().unwrap());
                        }
                    )*
                    return Poll::Ready(err.unwrap());
                } else {
                    Poll::Pending
                }
            }).await
        }
    } }
}