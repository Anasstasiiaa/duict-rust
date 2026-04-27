/// Library for demonstrating error handling and documentation in Rust.

/// Adds two numbers
///
/// # Examples
///
/// ```
/// let result = sr2::add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Safe division with error handling
///
/// # Errors
///
/// Returns error if division by zero occurs
pub fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}
