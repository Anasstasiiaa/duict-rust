# FilesIndex Workspace (PR3)

## Structure

- lib (core logic)
- main (CLI)

## Features

- File indexing by tags
- JSON storage
- Error handling:
  - thiserror (lib)
  - anyhow (cli)

## Usage

Set env:
FILES_INDEX_PATH=json:data.json

Add:
cargo run -- add --path file.txt --tags work

Get:
cargo run -- get --tags work

## Technologies

- Rust
- workspace
- thiserror
- anyhow
