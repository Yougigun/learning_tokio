use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[tokio::main]
async fn main() {
    let future = hello("world");
    future.await;
}

enum Hello {
    Init { name: String },
    Done,
}

impl Future for Hello {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match *self {
            Hello::Init { ref name } => println!("hello, {name}!"),
            Hello::Done => panic!("Please stop polling me!"),
        };
        *self = Hello::Done;
        Poll::Ready(())
    }
}

fn hello(name: &'static str) -> impl Future<Output = ()> {
    Hello::Init { name: name.into() }
}
struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            // Ignore this line for now.
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

use std::time::{Duration, Instant};
enum MainFuture {
    // Initialized, never polled
    State0,
    // Waiting on `Delay`, i.e. the `future.await` line.
    State1(Delay),
    // The future has completed.
    Terminated,
}

impl Future for MainFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        use MainFuture::*;
        loop {
            match *self {
                State0 => {
                    let when = Instant::now() + Duration::from_millis(10);
                    let future: Delay = Delay { when };
                    *self = State1(future);
                }
                State1(ref mut my_future) => match Pin::new(my_future).poll(cx) {
                    Poll::Ready(out) => {
                        assert_eq!(out, "done");
                        *self = Terminated;
                        return Poll::Ready(());
                    }
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                },
                Terminated => {
                    panic!("future polled after completion")
                }
            }
        }
    }
}


async fn hello_() {
    let delay = Delay { when: Instant::now() + Duration::from_millis(10) };
    let pinned_delay = Box::pin(delay);
    pinned_delay.await;
    println!("hello, world!");
}