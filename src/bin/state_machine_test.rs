use std::future::Future;
use std::pin::Pin;
use std::task::{Poll, Context};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;

#[derive(Debug, Clone)]
struct Condition { 
    ready: Arc<AtomicBool>,
}

impl Condition {
    fn new(initial: bool) -> Self {
        Self { ready: Arc::new(AtomicBool::new(initial)) }
    }

    fn set(&self, value: bool) {
        self.ready.store(value, Ordering::Release);
    }
}

impl Future for Condition {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
       if self.ready.load(Ordering::Acquire) {
            return Poll::Ready(());
       }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

enum StateMachine {
    State0 { cond: Condition },
    State1 { cond: Condition },
    Terminated,
}

impl StateMachine {
    fn new(cond: Condition) -> Self {
        StateMachine::State0 { cond }
    }
}

impl Future for StateMachine {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {

        loop {
           match &mut *self {

               StateMachine::State0 { cond } => { 
                    let cond = cond.clone();
                    *self = StateMachine::State1 { cond };
               }

               StateMachine::State1 { cond } => {
                   // poll the condition
                   let mut cond = cond.clone(); 
                   match Pin::new(&mut cond).poll(cx) {
                        Poll::Ready(()) => {
                            *self = StateMachine::Terminated;
                            return Poll::Ready(());
                        }
                        Poll::Pending => return Poll::Pending,
                   }
               }

               StateMachine::Terminated => panic!("future polled after completion!"),
           }
        }
    } 
}

#[tokio::main]
async fn main() {
    let cond = Condition::new(false);

    // spawn something that flips it
    let cond_c = cond.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        cond_c.set(true);
    });

    let s = StateMachine::new(cond);
    s.await;
}
