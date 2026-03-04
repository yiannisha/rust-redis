use mini_redis::{client, Result};
use futures::future::join_all;
use std::collections::HashMap;

async fn test() -> Result<()> {
    let m = HashMap::from([
        ("Mercury", 0.4),
        ("Venus", 0.7),
        ("Earth", 1.0),
        ("Mars", 1.5),
    ]);

    let mut futures = m.into_iter().map(|(k, v)| async move {
        let mut client = client::connect("127.0.0.1:6379").await?;
        client.set(k, v.to_string().into()).await?;
        println!("setting key: {:?} with value: {:?}", k, v);
        Ok::<(), mini_redis::Error>(())
    });

    println!("--- this should print before anything else ---");

    for r in join_all(futures).await { r?; }

    Ok(())

}

// #[tokio::main]
// async fn main() -> Result<()> {
//     test().await
// }

// non-macro version of tokio
fn main() -> Result<()> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        test().await
    })
}
