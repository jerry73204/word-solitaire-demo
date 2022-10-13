# Word Solitaire Game Server/Client in Rust Demo

This project a simple server and client to play word solitaire game
online. It was created for educational purpose to learn Rust
programming and async/.await.

## Installation

The Rust toolchain is required to build this project. Visit
[rustup.rs](https://rustup.rs/) to install the toolchain on your
system.

Run this cargo command to build the project. It will produce two
binaries at `target/debug` directory.

```bash
cargo build
```

Alternatively, you can add a `--release` option to build optimized
binaries. The programs run faster but with less debugging messages. It
will produce two binaries at `target/release` directory

```bash
cargo build --release
```
