# Practical Work 8

## Topic

Manual implementation of Rust Futures

## Goal

To understand how Rust async works internally by implementing the Future trait manually without using external runtimes.

---

## 8.1 Completed tasks

### 1. MeasurableFuture

- Implemented custom Future wrapper
- Measures execution time of inner future
- Logs execution duration

Особливості:

- Не використовується Unpin
- Використано unsafe Pin::new_unchecked для проєкції

Результат:
Продемонстровано роботу structural pinning та контроль переміщення даних у пам’яті.

---

### 2. Timer Future (Delay)

- Реалізовано Future, який завершується через заданий час

Особливості:

- Не блокує executor
- Використано окремий потік

Результат:
Імітовано поведінку реального async runtime (reactor pattern).

---

### 3. Custom Executor (block_on)

- Реалізовано власний executor

Особливості:

- Використано thread::park()
- Реалізовано власний Waker

Результат:
Показано як runtime “будить” задачі без busy waiting.

---

## Архітектура

- Future trait (ручна імплементація)
- Waker + Context
- Reactor (через потік)
- Executor (block_on)

---

## Технології

- Rust (std only)
- Без сторонніх крейтів

---

## Запуск

cargo run

---

## Очікуваний результат

Starting custom async runtime...
MeasurableFuture: Execution completed in ~1.5s
Runtime finished execution.

---

## Висновок

Робота демонструє, як працює async у Rust "під капотом":

- futures є lazy
- виконання керується executor
- пробудження задач відбувається через Waker
