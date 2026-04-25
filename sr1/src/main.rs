mod conversions;
mod macros_demo;
mod ownership;

fn main() {
    println!("=== Ownership & Borrowing ===");
    ownership::run();

    println!("\n=== Type Conversions ===");
    conversions::run();

    println!("\n=== Macros ===");
    macros_demo::run();
}
