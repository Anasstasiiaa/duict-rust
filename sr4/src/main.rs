use std::sync::{Arc, Mutex};
use tokio::time::{Duration, sleep};

// ---------------- TOKIO ASYNC ----------------
async fn tokio_task() {
    println!("Tokio task started");
    sleep(Duration::from_secs(1)).await;
    println!("Tokio task finished");
}

// ---------------- ASYNC-STD ----------------
async fn async_std_task() {
    println!("Async-std task started");
    async_std::task::sleep(std::time::Duration::from_secs(1)).await;
    println!("Async-std task finished");
}

// ---------------- CANCELLATION ----------------
async fn cancellable_task() {
    loop {
        println!("Working...");
        sleep(Duration::from_secs(1)).await;
    }
}

// ---------------- ACTOR MODEL ----------------
use tokio::sync::mpsc;

struct Actor {
    receiver: mpsc::Receiver<i32>,
}

impl Actor {
    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            println!("Actor received: {}", msg);
        }
    }
}

// ---------------- MAIN ----------------
#[tokio::main]
async fn main() {
    println!("=== Tokio Runtime ===");
    tokio::spawn(tokio_task()).await.unwrap();

    println!("\n=== Async-std Runtime ===");
    async_std::task::spawn(async_std_task()).await;

    println!("\n=== Task Cancellation ===");
    let handle = tokio::spawn(cancellable_task());

    sleep(Duration::from_secs(3)).await;
    handle.abort(); // відміна задачі
    println!("Task cancelled");

    println!("\n=== Actor Model ===");
    let (tx, rx) = mpsc::channel(10);

    let actor = Actor { receiver: rx };
    tokio::spawn(actor.run());

    tx.send(1).await.unwrap();
    tx.send(2).await.unwrap();

    sleep(Duration::from_secs(1)).await;
}
