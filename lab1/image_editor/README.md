# Lab 7 - Async + CPU separation

## Changes

- IO tasks moved to async (tokio + reqwest)
- CPU tasks moved to rayon
- no blocking main thread

## Result

Improved performance and responsiveness
