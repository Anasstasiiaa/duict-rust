# Lab 9 — Async Code Refactoring

## Goal

Improve async performance and code structure using Tokio and futures.

## Changes

- Replaced blocking HTTP with async reqwest
- Removed rayon, replaced with tokio::spawn
- Used join_all for concurrent execution
- Moved CPU-bound work into spawn_blocking

## Architecture

- IO-bound → async/await
- CPU-bound → blocking thread pool
- Tasks → tokio::spawn

## Result

Cleaner async pipeline and better scalability.

## Conclusion

Proper separation of IO and CPU tasks improves performance and prevents runtime blocking.
