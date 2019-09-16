use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

/// A stream joining two or more streams.
///
/// This stream is returned by `join!`.
#[derive(Debug)]
pub struct JoinStream<'a, L, R, T> {
    left: &'a mut L,
    right: &'a mut R,
    _marker: PhantomData<T>,
}

impl<L, R, T> Unpin for JoinStream<'_, L, R, T> {}

impl<'a, L, R, T> Stream for JoinStream<'a, L, R, T>
where
    L: Stream<Item = T> + Unpin,
    R: Stream<Item = T> + Unpin,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(Some(item)) = Pin::new(&mut *self.left).poll_next(cx) {
            // The first stream made progress. The JoinStream needs to be polled
            // again to check the progress of the second stream.
            cx.waker().wake_by_ref();
            Poll::Ready(Some(item))
        } else {
            Pin::new(&mut *self.right).poll_next(cx)
        }
    }
}
