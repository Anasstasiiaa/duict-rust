use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MyArc<T> {
    ptr: NonNull<Inner<T>>,
}

struct Inner<T> {
    ref_count: AtomicUsize,
    value: T,
}
