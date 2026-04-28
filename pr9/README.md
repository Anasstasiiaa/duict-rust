# Practical Work 9

## Topic

Using Rust async ecosystem for optimization

## Goal

Improve async code using futures, tokio tools

---

## 9.1 Completed tasks

### Refactoring PR7

Було:

- spawn для кожного URL
- без контролю кількості задач

Стало:

- futures::stream
- buffer_unordered для контролю concurrency

Результат:

- ефективніше використання ресурсів
- контроль навантаження

---

## Використані інструменти

- futures::stream
- tokio runtime
- async/await

---

## Запуск

cargo run -- urls.txt

---

## Висновок

Refactoring дозволив зробити код більш масштабованим і безпечним.
