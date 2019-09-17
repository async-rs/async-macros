use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

/// A stream joining two or more streams.
///
/// This stream is returned by `join!`.
#[derive(Debug)]
pub struct JoinStream<L, R> {
    left: L,
    right: R,
}

impl<L, R> Unpin for JoinStream<L, R> {}

impl<L, R> JoinStream<L, R> {
    #[doc(hidden)]
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

impl<L, R, T> Stream for JoinStream<L, R>
where
    L: Stream<Item = T> + Unpin,
    R: Stream<Item = T> + Unpin,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(Some(item)) = Pin::new(&mut self.left).poll_next(cx) {
            // The first stream made progress. The JoinStream needs to be polled
            // again to check the progress of the second stream.
            cx.waker().wake_by_ref();
            Poll::Ready(Some(item))
        } else {
            Pin::new(&mut self.right).poll_next(cx)
        }
    }
}

/// Combines multiple streams into a single stream of all their outputs.
///
/// This macro is only usable inside of async functions, closures, and blocks.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use async_macros::join_stream as join;
/// use futures::stream::{self, StreamExt};
/// use futures::future::ready;
///
/// let a = stream::once(ready(1u8));
/// let b = stream::once(ready(2u8));
/// let c = stream::once(ready(3u8));
///
/// let mut s = join!(a, b, c);
///
/// assert_eq!(s.next().await, Some(1u8));
/// assert_eq!(s.next().await, Some(2u8));
/// assert_eq!(s.next().await, Some(3u8));
/// assert_eq!(s.next().await, None);
/// # });
/// ```
#[macro_export]
macro_rules! join_stream {
    ($stream1:ident, $stream2:ident, $($stream:ident),* $(,)?) => {{
        let joined = $crate::JoinStream::new($stream1, $stream2);
        $(
            let joined = $crate::JoinStream::new(joined, $stream);
        )*
        joined
    }};
}
