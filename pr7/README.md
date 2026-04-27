# PR7 Async Web Downloader

## Features

- Async downloading
- CLI support
- Reads from file or stdin

## Usage

cargo run -- urls.txt

or

type urls.txt | cargo run

## Options

--max-threads=N

## Technologies

- tokio
- reqwest
- clap
