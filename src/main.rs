use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};

use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::hash::{DefaultHasher, Hasher, Hash};
use std::ptr::hash;

// `Arc` is a counted reference (pointer)
// `Mutex` is a **sync** mutex
// `Bytes` is an `Arc<Vec<u8>>` with some extra utility
type Db = Arc<Mutex<HashMap<String, Bytes>>>;

type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

fn new_sharded_db(num_shards: usize) -> ShardedDb {
    let mut db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(db)
}

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening...");

    let db = new_sharded_db(10);
    println!("created db of size: {}", db.len());

    // hash("test");
    let five = 5;
    let five_ref = &five;
    let mut hasher = DefaultHasher::new();
    hash(five_ref, &mut hasher);
    let h = hasher.finish();
    println!("{:?}", h);

    let shard = db[h as usize % db.len()].lock().unwrap();
    shard.insert("key", "value");
    println!("{:?}", db);

    return;

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();

        // clone the handle to the hash map.
        // this doesn't clone the db data but the `handle`, since this is an `Arc`
        let db = db.clone();

        // a new task is spawned for each inbound socket. the socket is
        // moved to the new task and processed there
        println!("Accepted");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    // The `Connection` lets us read/write redis **frames** instead of
    // byte streams. The `Connection` type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    // use `read_frame` to receive a command from the connection.
    while let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT {:?}", frame);

        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // the value is stored as `Bytes`
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }

            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    // `Frame::Bulk` expects data to be of type `Bytes`.
                    // `&Vec<u8>` is converted to `Bytes` using `into()`.
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }

            // cmd => panic!("unimplemented {:?}", cmd),
            cmd => {
                let s = format!("unimplemented {:?}", cmd);
                println!("{}", s);
                Frame::Error(s)
            }
        };
    
        // write the response to the client
        connection.write_frame(&response).await.unwrap();

        println!("{:?}", db);
    }
}
