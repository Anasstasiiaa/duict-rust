# Lab 3: Error Handling and Documentation

## Description

Refactored image editor project:

- Added structured error handling using `thiserror`
- Replaced String-based errors with enum
- Added Rust doc-comments
- Enabled strict lints

## Features

- Strong typed errors
- Automatic conversions via `From`
- Generated documentation via cargo doc

## Run

```bash
set MYME_UPLOADER=fs
set MYME_FILES_PATH=output

cargo run -- --files images.txt --resize 200x200
```

## Bug Tracking Systems Comparison

### Jira

- Powerful workflows
- Good for enterprise

* Complex setup

### GitHub Issues

- Free and integrated with Git
- Easy to use

* Limited advanced workflows

### Linear

- Fast and modern UI

* Paid for teams

### Bugzilla

- Very stable

* Old interface
