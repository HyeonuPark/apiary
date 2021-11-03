use apiary::{api, server::Server};
use async_trait::async_trait;
use std::sync::Arc;

#[api(server(FooServer))]
#[async_trait]
pub trait Foo: Send + Sync + 'static {
    #[get("/foo/{bar}/{baz}/quux")]
    async fn bar(self: Arc<Self>, bar: u32, baz: Result<bool, apiary::BoxError>) -> String;
}

struct Bar;

#[async_trait]
impl Foo for Bar {
    async fn bar(self: Arc<Self>, bar: u32, baz: Result<bool, apiary::BoxError>) -> String {
        format!("Hello HTTP! bar: {}, baz: {:?}", bar, baz)
    }
}

#[tokio::main]
async fn main() {
    FooServer(Arc::new(Bar))
        .bind("127.0.0.1:9000".parse().unwrap())
        .unwrap()
        .run()
        .await
        .unwrap();
}
