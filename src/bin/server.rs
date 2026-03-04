use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};

use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::hash::{DefaultHasher, Hasher, Hash};
use std::ptr::hash;
use std::ops::{Deref, Index};

const NUM_SHARDS: usize = 3; 

// `Arc` is a counted reference (pointer)
// `Mutex` is a **sync** mutex
// `Bytes` is an `Arc<Vec<u8>>` with some extra utility
type Db = Arc<Mutex<HashMap<String, Bytes>>>;

// a sharded version of `Db`
type ShardType = Mutex<HashMap<String, Bytes>>;

#[derive(Debug)]
struct Shards(Vec<ShardType>);

impl Shards {

    fn new(num_shards: usize) -> Self {
        let mut db = Vec::with_capacity(num_shards);
        for _ in 0..num_shards { db.push(Mutex::new(HashMap::new())); }
        Self(db)
    }

    fn shard_for(&self, key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        hash(key, &mut hasher);
        hasher.finish() as usize % self.0.len()
    }

}

impl Index<&str> for Shards {
    type Output = ShardType;

    fn index(&self, index: &str) -> &Self::Output {
        &self.0[self.shard_for(index)]
    }
}

type ShardedDb = Arc<Shards>;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening..."); 

    let db: ShardedDb = Arc::new(Shards::new(NUM_SHARDS));
    
    println!("{:?}", db["test"]);

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

async fn process(socket: TcpStream, db: ShardedDb) {
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
                let mut db = db[cmd.key()].lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }

            Get(cmd) => {
                let db = db[cmd.key()].lock().unwrap();
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
