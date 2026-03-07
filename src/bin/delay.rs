use tokio::net::TcpStream;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub(crate) struct Delay {
    pub(crate) when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            println!("Hello world");
            return Poll::Ready("done");
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_secs(10);
    let future = Delay { when };

    let out = future.await;
    assert_eq!(out, "done");
}
