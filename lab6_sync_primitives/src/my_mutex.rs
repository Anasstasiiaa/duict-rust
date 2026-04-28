use std::sync::atomic::{AtomicBool, Ordering};

pub struct MyMutex<T> {
    locked: AtomicBool,
    data: T,
}
