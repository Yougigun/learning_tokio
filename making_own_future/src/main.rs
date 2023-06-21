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
