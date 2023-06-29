use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}
// sharing state between thread and future
struct SharedState {
    // whether or not the sleep time has elapsed
    completed: bool,
    // waker for the task that `TimerFuture` is running on
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("TimerFuture::poll");
        // look at the shared state to see if the timer has already completed
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // set waker so that the thread can wake up the current task
            // when the timer has completed, ensuring that the future is polled
            // again and sees that `completed` is now `true`´
            println!("TimerFuture::pending");
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    // create a new `TimerFuture` which will complete after the provided
    // timeout.
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None, // no waker has been set
        }));

        // spawn the new thread
        let thread_shared_state = Arc::clone(&shared_state);
        thread::spawn(move || {
            thread::sleep(duration);

            let mut shared_state = thread_shared_state.lock().unwrap();
            // signal that the timer has completed and wake up the last
            // task on which the future was polled, if one exists.
            shared_state.completed = true;
            // if waker is none, then means future not return any thins yet
            if let Some(waker) = shared_state.waker.take() {
                waker.wake() // this is to wake up the task(put it back to the task queue)
            }
        });

        TimerFuture { shared_state }
    }
}
