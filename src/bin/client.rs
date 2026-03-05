// use mini_redis::client;
//
// #[tokio::main]
// fn main() {
//     // establish a connection to the server
//     let mut client = client::connect("127.0.0.1:6379").await.unwrap();
//
//     // spawn two tasks, one gets a key, the other sets a key
//     let t1 = tokio::spawn(async {
//         let res = client.get("foo").await;
//     });
//
//     let t2 = tokio::spawn(async {
//         client.set("foo", "bar".into()).await;
//     });
//
//     t1.await.unwrap();
//     t2.await.unwrap();
//
//
//     println!("test");
// }

fn main() {}
