use rand::Rng;
use rayon::prelude::*;
use std::fs;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};

// ================== PART 1 ==================
fn matrix_demo() {
    let (tx, rx) = mpsc::channel();

    // producer
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        let matrix: Vec<Vec<i32>> = (0..4096)
            .map(|_| (0..4096).map(|_| rng.gen_range(0..10)).collect())
            .collect();

        tx.send(matrix).unwrap();
    });

    // consumers
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let rx = rx.clone();
            thread::spawn(move || {
                let matrix = rx.recv().unwrap();

                let sum: i32 = matrix.par_iter().map(|row| row.iter().sum::<i32>()).sum();

                println!("Sum: {}", sum);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

// ================== PART 2 ==================
fn file_pipeline(path: &str) {
    let (tx, rx) = mpsc::channel();

    let counter = Arc::new(Mutex::new(0));

    // producer
    let tx_clone = tx.clone();
    let path = path.to_string();
    thread::spawn(move || {
        for entry in WalkDir::new(path) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                let content = fs::read(entry.path()).unwrap();
                tx_clone.send((entry.path().to_owned(), content)).unwrap();
            }
        }
    });

    // consumers (3 threads)
    for _ in 0..3 {
        let rx = rx.clone();
        let counter = Arc::clone(&counter);

        thread::spawn(move || {
            let key = Key::<Aes256Gcm>::from_slice(&[0u8; 32]);
            let cipher = Aes256Gcm::new(key);

            while let Ok((path, data)) = rx.recv() {
                let nonce = Nonce::from_slice(&[0u8; 12]);
                let encrypted = cipher.encrypt(nonce, data.as_ref()).unwrap();

                let new_path = path.with_extension("data");
                fs::write(new_path, encrypted).unwrap();

                let mut count = counter.lock().unwrap();
                *count += 1;
            }
        });
    }

    // monitor thread
    let counter_clone = Arc::clone(&counter);
    thread::spawn(move || {
        let mut last = 0;

        loop {
            let current = *counter_clone.lock().unwrap();
            if current != last {
                println!("Processed files: {}", current);
                last = current;
            }
        }
    });

    thread::sleep(std::time::Duration::from_secs(5));
}

fn main() {
    println!("--- Matrix demo ---");
    matrix_demo();

    println!("--- File pipeline ---");
    file_pipeline("test_folder");
}
