# PR5 Multithreading Rust

## Features

### Matrix processing

- Producer generates 4096x4096 matrix
- 2 threads compute sum (rayon)

### File pipeline

- Recursive file reading
- 3 worker threads encrypt files
- Shared counter (Mutex)
- Monitor thread prints progress

## Technologies

- threads
- channels
- Arc / Mutex
- rayon
- AES-256-GCM

## Run

cargo run
