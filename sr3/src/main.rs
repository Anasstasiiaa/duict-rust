use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crossbeam::channel;

// ---------------- RACE CONDITION (запобігання через Mutex) ----------------
fn race_condition_demo() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..5 {
        let c = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                let mut num = c.lock().unwrap();
                *num += 1;
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Counter result: {}", *counter.lock().unwrap());
}

// ---------------- DEADLOCK DEMO ----------------
fn deadlock_demo() {
    let lock1 = Arc::new(Mutex::new(0));
    let lock2 = Arc::new(Mutex::new(0));

    let l1 = Arc::clone(&lock1);
    let l2 = Arc::clone(&lock2);

    let _t1 = thread::spawn(move || {
        let _a = l1.lock().unwrap();
        thread::sleep(Duration::from_millis(100));
        let _b = l2.lock().unwrap();
    });

    let l1 = Arc::clone(&lock1);
    let l2 = Arc::clone(&lock2);

    let _t2 = thread::spawn(move || {
        let _b = l2.lock().unwrap();
        thread::sleep(Duration::from_millis(100));
        let _a = l1.lock().unwrap();
    });

    println!("Deadlock example created (threads may hang)");
}

// ---------------- CHANNEL (std) ----------------
fn std_channel_demo() {
    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        tx.send("Hello from thread").unwrap();
    });

    println!("Received: {}", rx.recv().unwrap());
}

// ---------------- CHANNEL (crossbeam) ----------------
fn crossbeam_demo() {
    let (tx, rx) = channel::unbounded();

    for i in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(i).unwrap();
        });
    }

    for _ in 0..3 {
        println!("Crossbeam received: {}", rx.recv().unwrap());
    }
}

// ---------------- NON-BLOCKING (Atomic) ----------------
use std::sync::atomic::{AtomicUsize, Ordering};

fn atomic_demo() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..5 {
        let c = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                c.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Atomic counter: {}", counter.load(Ordering::SeqCst));
}

// ---------------- MAIN ----------------
fn main() {
    println!("=== Race Condition (fixed with Mutex) ===");
    race_condition_demo();

    println!("\n=== Deadlock Demo ===");
    deadlock_demo();

    println!("\n=== std::channel ===");
    std_channel_demo();

    println!("\n=== crossbeam channel ===");
    crossbeam_demo();

    println!("\n=== Atomic (lock-free) ===");
    atomic_demo();
}
