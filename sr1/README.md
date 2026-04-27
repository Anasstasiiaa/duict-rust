# SR1 - Ownership, Conversions, Macros in Rust

## Ownership and Borrowing

Rust uses ownership model to guarantee memory safety without garbage collection.

### обходи системи

- Rc<T> — multiple ownership (single-thread)
- RefCell<T> — runtime borrow checking
- Arc<T> — thread-safe ownership
- Mutex<T> — safe mutable access in multithreading

These mechanisms do not break Rust guarantees, but move checks to runtime or use synchronization.

---

## Type Conversions

Implemented traits:

- From / Into
- TryFrom / TryInto
- AsRef / AsMut
- Borrow
- Deref / DerefMut

---

## Crates

- derive_more — reduces boilerplate for conversions

---

## Macros

### Declarative macros

- macro_rules!

### Procedural macros

- #[derive(Debug)]

Examples from std:

- println!
- vec!

---

## Limitations of macros

- hard to debug
- reduce readability
- complex syntax

### When not to use macros

- simple logic
- when function is enough
