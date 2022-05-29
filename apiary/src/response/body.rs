use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use http_body::SizeHint;
use pin_project::pin_project;

use crate::BoxError;

#[pin_project]
#[derive(Debug)]
pub struct Body {
    #[pin]
    repr: Repr,
}

#[pin_project(project = Proj)]
#[derive(Debug)]
enum Repr {
    Empty,
    Once(Bytes),
}

impl Body {
    pub fn empty() -> Self {
        Body { repr: Repr::Empty }
    }

    pub fn once<T: Into<Bytes>>(bytes: T) -> Self {
        let bytes = bytes.into();

        Body {
            repr: if bytes.is_empty() {
                // `Repr::Once()` SHOULD NOT contains empty body
                Repr::Empty
            } else {
                Repr::Once(bytes)
            },
        }
    }
}

impl http_body::Body for Body {
    type Data = Bytes;
    type Error = BoxError;

    fn poll_data(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match self.as_mut().project().repr.project() {
            Proj::Empty => Poll::Ready(None),
            Proj::Once(b) => {
                let b = mem::take(b);
                self.set(Body::empty());
                Poll::Ready(Some(Ok(b)))
            }
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        matches!(&self.repr, Repr::Empty)
    }

    fn size_hint(&self) -> SizeHint {
        match &self.repr {
            Repr::Empty => SizeHint::with_exact(0),
            Repr::Once(b) => SizeHint::with_exact(b.len() as u64),
        }
    }
}
