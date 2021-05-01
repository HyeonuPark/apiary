use apiary::apiary;
use async_trait::async_trait;
use std::sync::Arc;

#[apiary]
#[async_trait]
pub trait Foo {
    #[get("/foo/{bar}/{baz}/quux")]
    async fn bar(self: Arc<Self>, bar: u32, baz: bool) -> Result<String, String>;
}

struct Bar;

#[async_trait]
impl Foo for Bar {
    async fn bar(self: Arc<Self>, bar: u32, baz: bool) -> Result<String, String> {
        Ok(format!("Hello HTTP! bar: {}, baz: {}", bar, baz))
    }
}

#[tokio::main]
async fn main() {
    Arc::new(Bar)
        .to_router()
        .run("127.0.0.1:9000".parse().unwrap())
        .await
        .unwrap();
}
