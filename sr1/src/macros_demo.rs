// декларативний макрос
macro_rules! say_hello {
    () => {
        println!("Hello from macro!");
    };
}

pub fn run() {
    say_hello!();
    vec_macro_example();
    println_macro_example();
}

// приклад vec!
fn vec_macro_example() {
    let v = vec![1, 2, 3];
    println!("Vector: {:?}", v);
}

// приклад std макроса
fn println_macro_example() {
    println!("This is println! macro from std");
}
