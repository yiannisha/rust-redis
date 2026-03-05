use tokio::sync::{mpsc, oneshot};
use bytes::Bytes;
use mini_redis::client;

/* --- MESSAGING --- */

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

/// provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

/* ----- */

#[tokio::main]
async fn main() {
    // create a new channel with a capacity of at most 3
    let (tx, mut rx) = mpsc::channel(32);
    
    // the `move` keyword is used to **move** ownership of `rx` into the task.
    let manager = tokio::spawn(async move {
        // establish a connection to the server
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        // start receiving messages
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, resp } => {
                    let res = client.get(&key).await;
                    
                    // ignore errors
                    let _ = resp.send(res);
                }
                Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    
                    // ignore errors
                    let _ = resp.send(res);
                }
            }
        }
    });

    // the `Sender` handles are moved into the tasks. as there are two
    // tasks, we need a second `Sender`.
    let tx2 = tx.clone();

    // spawn two tasks, one gets a key, the other sets a key
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };

        // send the GET request
        tx.send(cmd).await.unwrap();

        // await the response
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        };

        // send the SET request
        tx2.send(cmd).await.unwrap();

        // await the response
        let res = resp_rx.await;
        println!("GOT = {:?}", res);

        let (resp_tx2, resp_rx2) = oneshot::channel();

        let cmd2 = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx2,
        };

        tx2.send(cmd2).await.unwrap();

        // await the response
        let res2 = resp_rx2.await;
        println!("GOT = {:?}", res2);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
