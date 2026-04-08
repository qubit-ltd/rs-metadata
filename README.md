# Qubit Metadata

[![CircleCI](https://circleci.com/gh/qubit-ltd/rust-metadata.svg?style=shield)](https://circleci.com/gh/qubit-ltd/rust-metadata)
[![Coverage Status](https://coveralls.io/repos/github/qubit-ltd/rust-metadata/badge.svg?branch=main)](https://coveralls.io/github/qubit-ltd/rust-metadata?branch=main)
[![Crates.io](https://img.shields.io/crates/v/qubit-metadata.svg?color=blue)](https://crates.io/crates/qubit-metadata)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

A general-purpose, type-safe, extensible metadata model for Rust.

## Overview

`qubit-metadata` provides a `Metadata` type — a structured key-value store
designed for any domain that needs to attach arbitrary, typed, serializable
annotations to its data models. Common use cases include:

- Attaching contextual information to domain entities (messages, records, events)
- Carrying provider- or adapter-specific fields without polluting core models
- Expressing query filter conditions over annotated data (e.g. vector stores)
- Passing through opaque metadata across service or library boundaries

It is not a plain `HashMap<String, String>`. It is a domain-level extensibility
point with type-safe access, `serde_json::Value` backing, and first-class
`serde` support.

## Design Goals

- **Type Safety**: Typed get/set API backed by `serde_json::Value`
- **Generality**: No domain-specific assumptions — usable in any Rust project
- **Extensibility**: Acts as a structured extension point, not a stringly-typed bag
- **Serialization**: First-class `serde` support for JSON interchange
- **Filtering**: Optional `MetadataFilter` for composable query conditions

## Features

### 🗂 **`Metadata`**

- Ordered key-value store with `String` keys and `serde_json::Value` values
- Two typed access layers: convenience `get::<T>()` / `set()` and explicit `try_get::<T>()` / `try_set()`
- Lightweight JSON kind inspection via `MetadataValueKind`
- Merge, extend, and iterate operations
- Full `serde` serialization / deserialization support
- `Debug`, `Clone`, `PartialEq`, `Default` derives

### 🔍 **`MetadataFilter`** *(optional, planned)*

- Composable filter expressions for metadata-based queries
- Supports equality, range, and logical combinators (`and`, `or`, `not`)
- Useful for filtering annotated records in databases, vector stores, or
  in-memory collections

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
qubit-metadata = "0.1.0"
```

## Usage

```rust
use qubit_metadata::Metadata;

let mut meta = Metadata::new();
meta.set("author", "alice");
meta.set("priority", 3_i64);
meta.set("reviewed", true);

// Convenience API: terse and forgiving.
let author: Option<String> = meta.get("author");
assert_eq!(author.as_deref(), Some("alice"));

// Explicit API: preserves failure reasons.
let priority = meta.try_get::<i64>("priority").unwrap();
assert_eq!(priority, 3);
```

## Error Handling

When callers need to distinguish missing keys from type mismatches, prefer the
explicit APIs and inspect `MetadataError`:

```rust
use qubit_metadata::{Metadata, MetadataError, MetadataValueKind};

let mut meta = Metadata::new();
meta.set("answer", "forty-two");

match meta.try_get::<i64>("answer") {
    Err(MetadataError::DeserializationError { actual, .. }) => {
        assert_eq!(actual, MetadataValueKind::String);
    }
    other => panic!("unexpected result: {other:?}"),
}
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
