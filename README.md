# S3 ML Uploader

A Rust-based utility for uploading files to AWS S3, S3-compatible storage (e.g., MinIO), and via direct HTTP with AWS Signature V4, enriched with a simple ML-driven file type classification.

---

## Table of Contents

1. [Description](#description)
2. [Features](#features)
3. [Architecture & Workflow](#architecture--workflow)
4. [Prerequisites](#prerequisites)
5. [Installation](#installation)
6. [Configuration](#configuration)
7. [Usage](#usage)
   - [Generating Test Files](#generating-test-files)
   - [Running the Uploader](#running-the-uploader)
8. [Code Structure](#code-structure)
9. [Environment Variables](#environment-variables)
10. [File Type Predictor](#file-type-predictor)
11. [Uploading Methods](#uploading-methods)
12. [Dependencies](#dependencies)
13. [Contributing](#contributing)
14. [License](#license)

---

## Description

`s3-ml-uploader` is a command-line tool written in Rust that processes a set of files, uses a lightweight ML model to classify each file into types (e.g., `images`, `documents`, `text`, `archives`, or `misc`), and uploads them concurrently to:

- **AWS S3** via the official AWS SDK
- **S3-compatible endpoints** (e.g., MinIO) using the `rust-s3` crate
- **Direct HTTP PUT** to AWS S3 with AWS Signature Version 4

It demonstrates parallel processing with `tokio`, simplistic ML heuristics, and multiple upload strategies.

## Features

- 📂 **ML-driven routing**: Classify files into folders based on content signatures.
- ⛓️ **Multiple upload backends**: AWS SDK, Rust-S3 (MinIO), and raw HTTP with V4 signing.
- 🔀 **Concurrent uploads**: Utilize `tokio::task` for parallelism.
- 🔄 **Extensible**: Drop-in replacement for the ML model or storage backends.

## Architecture & Workflow

```text
  +----------------+      +-----------------+       +--------------+
  | file1.txt,...  | ---> | ML Predictor   | --->  | S3 Folder Key|
  +----------------+      +-----------------+       +--------------+
             |                                           |
             v                                           v
       Processing Loop                            Upload Tasks:
     (tokio::spawn for each file)                 ├→ AWS SDK
                                                  ├→ MinIO SDK
                                                  └→ HTTP PUT V4
```

1. **Process**: Read each file, predict its type (e.g., `images/foo.png`).  
2. **Upload**: Spawn three asynchronous tasks per file:
   - AWS S3 via `aws-sdk-s3`
   - MinIO via `rust-s3`
   - HTTP PUT with Signature V4 via `reqwest`
3. **Report** upon completion of all tasks.

## Prerequisites

- Rust toolchain (Rust 1.60+)
- AWS credentials with S3 permissions
- For S3-compatible storage (e.g., MinIO): running endpoint
- `bash` for the test files script

## Installation

```bash
# Clone repository
git clone https://github.com/nicktretyakov/s3-ml-uploader.git
cd s3-ml-uploader
# Build in release mode
cargo build --release
```

## Configuration

Set the following environment variables (can be placed in a `.env` file for convenience):

| Variable         | Description                                        | Default                            |
|------------------|----------------------------------------------------|------------------------------------|
| `AWS_ACCESS_KEY` | AWS access key ID                                  | (none; must be provided)           |
| `AWS_SECRET_KEY` | AWS secret access key                              | (none; must be provided)           |
| `AWS_REGION`     | AWS region for S3                                  | `us-east-1`                        |
| `AWS_BUCKET`     | Target S3 bucket name                              | `aws-bucket`                       |
| `S3_ACCESS_KEY`  | Access key for S3-compatible storage (MinIO)       | `minioadmin`                       |
| `S3_SECRET_KEY`  | Secret key for S3-compatible storage               | `minioadmin`                       |
| `S3_ENDPOINT`    | URL of S3-compatible service (HTTP)                | `http://localhost:9000`            |
| `S3_BUCKET`      | Bucket name on S3-compatible endpoint              | `minio-bucket`                     |

## Usage

### Generating Test Files

A helper script `create-test-files.sh` generates three sample text files in the working directory:

```bash
# Ensure script is executable
chmod +x create-test-files.sh
# Run to create file1.txt, file2.txt, file3.txt
./create-test-files.sh
```

### Running the Uploader

With your env vars set (or `.env` loaded via `dotenv`), execute:

```bash
# Using dotenv for .env auto-loading (requires dotenv binary or cargo script)
dotenv -- cargo run --release
```

Output will indicate classification and upload status for each file.

## Code Structure

```
├── Cargo.toml         # Dependencies and metadata
├── create-test-files.sh  # Test data generator
├── src/
│   ├── main.rs       # Entry point: orchestrates ML prediction and uploads
│   └── ml.rs         # `FileTypePredictor`: simple signature heuristics
└── .env.example      # Template for environment variables
```

## Environment Variables

- `.env` file is supported by `dotenv` crate.
- Variables override defaults at runtime.

## File Type Predictor

Located in `src/ml.rs`, `FileTypePredictor` uses hard-coded byte signatures for common formats:

- **PDF** (`%PDF`)
- **JPEG**, **PNG**, **GIF** → `images`
- **ZIP** → `archives`
- Fallback: checks if >80% of first 1KB is printable → `text`, else `misc`.

This can be replaced with a real ML model (e.g., ONNX, TensorFlow).

## Uploading Methods

1. **AWS SDK (`aws-sdk-s3`)**
2. **Rust-S3 crate** for S3-compatible storages
3. **Direct HTTP PUT** with AWS Signature V4 via `reqwest`

Each is demonstrated to show different integration approaches in Rust.

## Dependencies

Key crates in `Cargo.toml`:

- `aws-sdk-s3`, `aws-config`  
- `rust-s3`  
- `reqwest`  
- `tokio`, `futures`, `rayon`  
- `hmac`, `sha2`, `hex` for signing
- `dotenv`, `chrono`, `base64`

## Contributing

Contributions welcome! Please open issues or PRs.  
Ensure adherence to Rust 2021 edition and include tests for new functionality.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

