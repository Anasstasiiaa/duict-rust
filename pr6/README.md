# Practical Work 6

## Topic: Send and Sync traits in Rust

## Goal

Understand the purpose of Send and Sync traits, why they exist, and why some types do not implement them.

---

## 6.1 Research

### Types that are !Send

Examples:

- Rc<T>
- *const T / *mut T (raw pointers)

Reason:

- Rc<T> is not thread-safe because it uses non-atomic reference counting
- Raw pointers have no guarantees of safety across threads

Conclusion:
!Send types cannot be safely transferred between threads because they may cause data races or undefined behavior.

---

### Types that are !Sync

Examples:

- RefCell<T>
- Cell<T>
- Rc<T>

Reason:

- RefCell and Cell allow interior mutability without synchronization
- Rc is not thread-safe

Conclusion:
!Sync types cannot be safely shared between threads via references.

---

## 6.2 Answers

### 1. What is Send?

Send means a type can be transferred between threads safely.

### 2. What is Sync?

Sync means a type can be safely shared between threads via references.

### 3. Difference between Send and Sync

- Send → ownership transfer
- Sync → shared access via references

### 4. Why do they exist?

They provide compile-time guarantees of thread safety.

### 5. Why some types do not implement them?

Because they:

- use unsafe memory access
- have no synchronization
- could cause data races

---

## Conclusion

Send and Sync are core traits that enforce thread safety in Rust. They prevent unsafe concurrency at compile time.
