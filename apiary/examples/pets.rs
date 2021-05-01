use apiary::apiary;
// use async_trait::async_trait;
use std::sync::Arc;

#[apiary]
// #[async_trait]
pub trait Foo {
    #[get("/foo/{bar}")]
    async fn bar(self: Arc<Self>, bar: u32) -> Result<String, String>;
}

fn main() {}
