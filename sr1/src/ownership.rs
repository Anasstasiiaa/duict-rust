use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn run() {
    rc_example();
    refcell_example();
    arc_example();
    mutex_example();
}

// Rc<T> — кілька власників (single-thread)
fn rc_example() {
    let a = Rc::new(10);
    let b = Rc::clone(&a);

    println!("Rc values: {}, {}", a, b);
}

// RefCell<T> — runtime borrow checking
fn refcell_example() {
    let data = RefCell::new(5);

    *data.borrow_mut() += 1;

    println!("RefCell value: {}", data.borrow());
}

// Arc<T> — thread-safe Rc
fn arc_example() {
    let data = Arc::new(20);
    let d = Arc::clone(&data);

    thread::spawn(move || {
        println!("Arc from thread: {}", d);
    })
    .join()
    .unwrap();
}

// Mutex<T> — safe mutable access між потоками
fn mutex_example() {
    let data = Arc::new(Mutex::new(0));
    let d = Arc::clone(&data);

    thread::spawn(move || {
        let mut val = d.lock().unwrap();
        *val += 1;
    })
    .join()
    .unwrap();

    println!("Mutex value: {}", *data.lock().unwrap());
}
