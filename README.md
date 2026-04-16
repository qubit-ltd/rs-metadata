# Qubit Metadata

[![CircleCI](https://circleci.com/gh/qubit-ltd/rs-metadata.svg?style=shield)](https://circleci.com/gh/qubit-ltd/rs-metadata)
[![Coverage Status](https://coveralls.io/repos/github/qubit-ltd/rs-metadata/badge.svg?branch=main)](https://coveralls.io/github/qubit-ltd/rs-metadata?branch=main)
[![Crates.io](https://img.shields.io/crates/v/qubit-metadata.svg?color=blue)](https://crates.io/crates/qubit-metadata)
[![Rust](https://img.shields.io/badge/rust-1.93+-blue.svg?logo=rust)](https://www.rust-lang.org)
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
- **Filtering**: `MetadataFilter` for composable predicates over `Metadata`

## Features

### 🗂 **`Metadata`**

- Ordered key-value store with `String` keys and `serde_json::Value` values
- Two typed access layers: convenience `get::<T>()` / `set()` and explicit `try_get::<T>()` / `try_set()`
- Lightweight JSON value type inspection via `MetadataValueType`
- Merge, extend, and iterate operations
- Full `serde` serialization / deserialization support
- `Debug`, `Clone`, `PartialEq`, `Default` derives

### 🔍 **`MetadataFilter`**

- Composable tree of conditions over metadata keys, evaluated with `matches(&metadata)`
- Comparison, existence, and set-membership leaves; logical `and`, `or`, and `not`
- `Serialize` / `Deserialize` for persisting or exchanging filter definitions as JSON
- Useful for in-memory filtering, query predicates, or any pipeline that needs a portable metadata predicate

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
qubit-metadata = "0.2.0"
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

## MetadataFilter

`MetadataFilter` describes a predicate on a single [`Metadata`](https://docs.rs/qubit-metadata/latest/qubit_metadata/struct.Metadata.html) value. You build a tree of conditions, then call [`matches`](https://docs.rs/qubit-metadata/latest/qubit_metadata/enum.MetadataFilter.html#method.matches) to test whether a given `Metadata` satisfies it.

### Leaf constructors

Each leaf inspects one key (values are serialized to [`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html) the same way as `Metadata::set`):

| Constructor | Semantics |
|-------------|-----------|
| `equal`, `not_equal` | Equality / inequality of the stored JSON value |
| `greater`, `greater_equal`, `less`, `less_equal` | Ordered comparison (numeric ordering, string lexicographic order, and mixed-type rules as implemented for your value shapes) |
| `exists`, `not_exists` | Key present / absent |
| `in_values`, `not_in_values` | Membership of the key’s value in a finite set |

### Logical combinators

- **`and`** — all sub-filters must match. If `self` is already an `And` node, the new filter is appended (flat structure).
- **`or`** — at least one sub-filter must match (same flattening behavior as `and`).
- **`not`** — negates a filter. The `!` operator is also implemented via [`Not`](https://doc.rust-lang.org/std/ops/trait.Not.html) for `MetadataFilter`.

Edge cases: an `And` with **no** children matches **every** `Metadata`; an `Or` with **no** children matches **none**.

### Evaluation

Call `filter.matches(&meta)` to obtain a `bool`. How a missing key interacts with each leaf operator is defined on [`Condition`](https://docs.rs/qubit-metadata/latest/qubit_metadata/enum.Condition.html).

By default, missing keys satisfy `not_equal` and `not_in_values` (legacy behavior). If you want strict behavior, call [`matches_with_missing_key_policy`](https://docs.rs/qubit-metadata/latest/qubit_metadata/enum.MetadataFilter.html#method.matches_with_missing_key_policy) with [`MissingKeyPolicy::NoMatch`](https://docs.rs/qubit-metadata/latest/qubit_metadata/enum.MissingKeyPolicy.html).

For numeric range predicates, mixed integer/float comparisons default to conservative precision-preserving behavior. Call [`matches_with_policies`](https://docs.rs/qubit-metadata/latest/qubit_metadata/enum.MetadataFilter.html#method.matches_with_policies) with [`NumberComparisonPolicy::Approximate`](https://docs.rs/qubit-metadata/latest/qubit_metadata/enum.NumberComparisonPolicy.html) when you prefer lossy-but-comparable fallback semantics.

### Serde

`MetadataFilter` implements `Serialize` and `Deserialize`, so you can store filters in configuration, databases, or JSON APIs alongside your metadata model.

### Examples

**AND — all conditions must hold:**

```rust
use qubit_metadata::{Metadata, MetadataFilter};

let mut meta = Metadata::new();
meta.set("status", "active");
meta.set("score", 42_i64);

let filter = MetadataFilter::equal("status", "active")
    .unwrap()
    .and(MetadataFilter::greater_equal("score", 10_i64).unwrap());

assert!(filter.matches(&meta));
```

**OR, membership, and negation:**

```rust
use qubit_metadata::{Metadata, MetadataFilter};

let mut meta = Metadata::new();
meta.set("region", "eu");
meta.set("tier", "pro");

let filter = MetadataFilter::in_values("region", ["eu", "us"])
    .unwrap()
    .or(MetadataFilter::equal("tier", "enterprise").unwrap());

assert!(filter.matches(&meta));

let hide_drafts = MetadataFilter::equal("status", "draft").unwrap().not();
// Equivalent: !MetadataFilter::equal("status", "draft").unwrap()
assert!(hide_drafts.matches(&meta));
```

## Error Handling

When callers need to distinguish missing keys from type mismatches, prefer the
explicit APIs and inspect `MetadataError`:

```rust
use qubit_metadata::{Metadata, MetadataError, MetadataValueType};

let mut meta = Metadata::new();
meta.set("answer", "forty-two");

match meta.try_get::<i64>("answer") {
    Err(MetadataError::DeserializationError { actual, .. }) => {
        assert_eq!(actual, MetadataValueType::String);
    }
    other => panic!("unexpected result: {other:?}"),
}
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
