use tower::layer::util::Stack;
use tower::ServiceBuilder;

pub mod limit_content_length;
pub mod stringify_body;

#[derive(Debug, thiserror::Error)]
#[error("Internal server error")]
pub struct PolledAfterComplete;

pub trait ServiceBuilderExt<L> {
    fn stringify_body(self) -> ServiceBuilder<Stack<stringify_body::Layer, L>>;
    fn limit_content_length(
        self,
        limit: usize,
    ) -> ServiceBuilder<Stack<limit_content_length::Layer, L>>;
}

impl<L> ServiceBuilderExt<L> for ServiceBuilder<L> {
    fn stringify_body(self) -> ServiceBuilder<Stack<stringify_body::Layer, L>> {
        self.layer(stringify_body::Layer)
    }

    fn limit_content_length(
        self,
        limit: usize,
    ) -> ServiceBuilder<Stack<limit_content_length::Layer, L>> {
        self.layer(limit_content_length::Layer::new(limit))
    }
}
