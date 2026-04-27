use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::ops::{Deref, DerefMut};

use derive_more::From;

pub fn run() {
    from_into();
    try_from();
    try_into_example();
    as_ref_example();
    as_mut_example();
    borrow_example();
    deref_example();
    deref_mut_example();
    derive_more_example();
}

// From / Into
fn from_into() {
    let s1 = String::from("hello");
    let s2: String = "world".into();

    println!("From/Into: {} {}", s1, s2);
}

// TryFrom
fn try_from() {
    let num = i32::try_from(10u8).unwrap();
    println!("TryFrom: {}", num);
}

// TryInto
fn try_into_example() {
    let x: i32 = 10u8.try_into().unwrap();
    println!("TryInto: {}", x);
}

// AsRef
fn as_ref_example() {
    fn print_str(s: impl AsRef<str>) {
        println!("AsRef: {}", s.as_ref());
    }

    print_str("hello");
}

// AsMut
fn as_mut_example() {
    let mut s = String::from("hello");

    fn make_uppercase(s: &mut impl AsMut<str>) {
        let s = s.as_mut();
        println!("AsMut: {}", s.to_uppercase());
    }

    make_uppercase(&mut s);
}

// Borrow
fn borrow_example() {
    let s = String::from("hello");
    let b: &str = s.borrow();

    println!("Borrow: {}", b);
}

// Deref
struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn deref_example() {
    let x = MyBox(5);
    println!("Deref: {}", *x);
}

// DerefMut
struct MyBoxMut<T>(T);

impl<T> Deref for MyBoxMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for MyBoxMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn deref_mut_example() {
    let mut x = MyBoxMut(10);
    *x += 5;
    println!("DerefMut: {}", *x);
}

// derive_more
#[derive(From)]
struct MyInt(i32);

fn derive_more_example() {
    let x: MyInt = 5.into();
    println!("derive_more: {}", x.0);
}
