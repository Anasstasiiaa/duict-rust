# Files Index CLI (PR2)

## Description

CLI application for indexing and searching files by tags.

## Features

- Add file with tags
- Search files by tags
- Supports two storage types:
  - JSON
  - SQLite

## Usage

### Add file:

cargo run -- add --path test.txt --tags work,study

### Search:

cargo run -- get --tags work

## Configuration

Set environment variable:

JSON:
FILES_INDEX_PATH=json:data.json

SQLite:
FILES_INDEX_PATH=sqlite:data.db

## Technologies

- Rust
- clap
- serde / serde_json
- rusqlite

## Polymorphism

Dynamic:

- Box<dyn Storage>

Static:

- trait Storage implementations

## Project structure

- storage.rs (trait)
- json_storage.rs
- sqlite_storage.rs
- main.rs
