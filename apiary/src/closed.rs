use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_channel::oneshot;
use pin_project::pin_project;

/// Async functions can be halted silently when its future got dropped before completion.
/// The Hyper server does so when the backing connection is closed
/// since in this case there's no way to send the result to the client.
///
/// But the Apiary runs the user handler until its completion even in this case.
/// To cancel the execution when the connection is closed, you can await this future
/// and returns whatever value since it will be discarded anyway.
#[pin_project]
#[derive(Debug)]
pub struct ConnectionClosed {
    #[pin]
    receiver: oneshot::Receiver<()>,
}

#[derive(Debug)]
pub struct ConnectionCloseGuard {
    sender: oneshot::Sender<()>,
}

impl ConnectionClosed {
    pub fn new() -> (ConnectionClosed, ConnectionCloseGuard) {
        let (sender, receiver) = oneshot::channel();
        (
            ConnectionClosed { receiver },
            ConnectionCloseGuard { sender },
        )
    }
}

impl Future for ConnectionClosed {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        const MSG: &str = "Oneshot channel inside of the connection closed future \
            should never send the normal message";
        self.project()
            .receiver
            .poll(cx)
            .map(|res| debug_assert!(matches!(res, Err(_)), MSG))
    }
}
