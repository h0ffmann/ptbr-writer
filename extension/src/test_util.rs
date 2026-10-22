//! Runs the crate's futures on the host: they never wait on a timer or a socket, only on a fake
//! whose state a test flips, so polling in a loop with a waker that does nothing is enough.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

pub fn block_on<F: Future>(f: F) -> F::Output {
    let mut f = std::pin::pin!(f);
    loop {
        if let Poll::Ready(v) = f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
            return v;
        }
    }
}

/// One poll: `None` while pending.
pub fn poll_once<F: Future>(f: &mut Pin<Box<F>>) -> Option<F::Output> {
    match f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(v) => Some(v),
        Poll::Pending => None,
    }
}
