#![allow(dead_code)]

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};
#[tokio::main]
async fn main() {
    let (resp, time) = TimedWrapper::new(reqwest::get("http://adamchalmers.com")).await;
    let resp = resp.unwrap();
    println!(
        "Got a HTTP Path:{}, HTTP Code {} in {}ms",
        resp.url(),
        resp.status(),
        time.as_millis()
    );
    let (resp, time) = TimedWrapper::new(reqwest::get("http://google.com")).await;
    let resp = resp.unwrap();
    println!(
        "Got a HTTP Path:{}, HTTP Code {} in {}ms",
        resp.url(),
        resp.status(),
        time.as_millis()
    )
}

/// A future which returns a random number when it resolves.
#[derive(Default)]
struct RandFuture;

impl Future for RandFuture {
    // Every future has to specify what type of value it returns when it resolves.
    // This particular future will return a u16.
    type Output = u16;

    // The `Future` trait has only one method, named "poll".
    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(rand::random())
    }
}

#[pin_project::pin_project] // This generates a `project` method
pub struct TimedWrapper<Fut: Future> {
    // For each field, we need to choose whether `project` returns an
    // unpinned (&mut T) or pinned (Pin<&mut T>) reference to the field.
    // By default, it assumes unpinned:
    start: Option<Instant>,
    // Opt into pinned references with this attribute:
    #[pin]
    future: Fut,
}
impl<Fut: Future> TimedWrapper<Fut> {
    pub fn new(future: Fut) -> Self {
        Self {
            future,
            start: None,
        }
    }
}

impl<Fut: Future> Future for TimedWrapper<Fut> {
    // This future will output a pair of values:
    // 1. The value from the inner future
    // 2. How long it took for the inner future to resolve
    type Output = (Fut::Output, Duration);

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // This returns a type with all the same fields, with all the same types,
        // except that the fields defined with #[pin] will be pinned.
        let this = self.project();
        // Call the inner poll, measuring how long it took.
        let start = this.start.get_or_insert_with(Instant::now);
        let inner_poll = this.future.poll(cx);
        let elapsed = start.elapsed();

        match inner_poll {
            // The inner future needs more time, so this future needs more time too
            Poll::Pending => Poll::Pending,
            // Success!
            Poll::Ready(output) => Poll::Ready((output, elapsed)),
        }
    }
}

#[test]
fn test() {
    #[derive(Copy, Clone)]
    struct sr<'a> {
        a: u32,
        b: &'a u32,
    }
    let mut n = 1;
    n = 3;
    let s = sr { a: n, b: &n };
    let b = s;

    println!("{} {}", s.a, s.b);
    println!("{} {}", b.a, b.b);
}
