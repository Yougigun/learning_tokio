use std::{future::Future, pin::Pin, thread::JoinHandle};

use tokio::{runtime::Runtime, task::JoinError};

fn main() {
    // Create the runtime
    let rt = Runtime::new().unwrap();
    // let tasks: Vec<Box<dyn Future<Output = Result<(), JoinError>>>> = Vec::new();
    let mut tasks: Vec<Pin<Box<dyn Future<Output = Result<(), JoinError>>>>> = Vec::new();
    // it is like goroutine
    let task1 = rt.spawn(async {
        println!("this is task1");
    });

    let task2 = rt.spawn(async move {
        println!("this is task2");
    });
    tasks.push(Box::pin(task1));
    tasks.push(Box::pin(task2));
    // Perform other non-blocking tasks in main thread
    println!("This will be printed before the async function");

    // Cleanup and wait for all spawned tasks to finish
    rt.block_on(async {
        for task in tasks {
            let _ = task.await;
        }
    });
}
