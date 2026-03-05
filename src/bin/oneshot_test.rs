use tokio::sync::oneshot;
use bytes::Bytes;

/// provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    // let (tx, rx) = oneshot::channel(); 
}
