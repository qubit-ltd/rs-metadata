# Qubit Metadata

[![CircleCI](https://circleci.com/gh/qubit-ltd/rs-metadata.svg?style=shield)](https://circleci.com/gh/qubit-ltd/rs-metadata)
[![Coverage Status](https://coveralls.io/repos/github/qubit-ltd/rs-metadata/badge.svg?branch=main)](https://coveralls.io/github/qubit-ltd/rs-metadata?branch=main)
[![Crates.io](https://img.shields.io/crates/v/qubit-metadata.svg?color=blue)](https://crates.io/crates/qubit-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

A general-purpose, type-safe metadata model for Rust.

## Overview

`qubit-metadata` provides a `Metadata` type for attaching explicitly typed
extension fields to data objects without hard-coding every auxiliary field into
the core model. Typical uses include:

- Document ingestion: keep `file_id`, `chunk_index`, `language`, `source`, and `confidence` with each document chunk.
- Vector search: store fields such as `tenant_id`, `doc_type`, `created_at`, `score`, or `acl_group` that later become vector-database metadata columns or payload filters.
- Event and message pipelines: carry `trace_id`, `request_id`, `tenant_id`, `route`, and retry metadata through services.
- External service integration: retain compact service result fields such as model version, latency, billing tag, and request ID for diagnostics and analytics.

`Metadata` stores values as `qubit_value::Value`, so scalar types remain clear:
`i64` is different from `u32`, and `f64` is different from `String`. If a caller
really needs nested data, it can store `Value::Json` explicitly. Common document
metadata, vector-database metadata, and request context are usually flat typed
field sets, which keeps schema validation and filter construction predictable.

## Design Goals

- **Typed values**: preserve concrete runtime types with `qubit_value::Value`.
- **Convenient construction**: support both mutable `set()` and fluent `with()`.
- **Optional schema**: validate field names, required fields, concrete `DataType`s, and filter compatibility.
- **Filter builder**: build immutable `MetadataFilter` values with a chainable builder.
- **Serde support**: serialize metadata, schemas, and filters for config, storage, and service boundaries.

## Features

### 1) Typed metadata storage

`Metadata` is an ordered `String -> Value` map. It supports typed `get`, `set`,
`try_get`, schema-checked `set_checked` / `with_checked`, fluent `with`,
iteration, merge, retain, and conversion to/from `BTreeMap<String, Value>`.

```rust
use qubit_metadata::Metadata;

let meta = Metadata::new()
    .with("author", "alice")
    .with("priority", 3_i64)
    .with("reviewed", true);

assert_eq!(meta.get::<String>("author").as_deref(), Some("alice"));
assert_eq!(meta.try_get::<i64>("priority").unwrap(), 3);
```

### 2) Schema for validation and storage planning

`MetadataSchema` uses `qubit_datatype::DataType`. This is useful when a storage
backend requires metadata fields to be declared in advance, and it also lets the
filter builder validate field/operator compatibility early. Its
`UnknownFieldPolicy` applies to both metadata validation and filter validation:
declared fields are still checked strictly, while unknown filter fields are
accepted only when the policy is `Allow`.

```rust
use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataSchema};

let schema = MetadataSchema::builder()
    .required("tenant_id", DataType::String)
    .required("score", DataType::Int64)
    .optional("source", DataType::String)
    .build();

let meta = Metadata::new()
    .with("tenant_id", "acme")
    .with("score", 42_i64);

schema.validate(&meta).unwrap();
```

### 3) Immutable filters built by builder

`MetadataFilter::builder()` creates a builder. Calling `build()` returns a
`Result<MetadataFilter, MetadataError>` so structurally invalid expressions such
as empty grouped expressions are reported instead of silently becoming no-ops.
`build_checked(&schema)` also validates referenced fields, operator
compatibility, and filter value types.

```rust
use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataFilter, MetadataSchema};

let schema = MetadataSchema::builder()
    .required("status", DataType::String)
    .required("score", DataType::Int64)
    .build();

let filter = MetadataFilter::builder()
    .eq("status", "active")
    .and_ge("score", 10)
    .build_checked(&schema)
    .unwrap();

let meta = Metadata::new()
    .with("status", "active")
    .with("score", 42_i64);

assert!(filter.matches(&meta));
```

### 4) Filter DSL

| Method | Semantics |
|--------|-----------|
| `eq`, `ne` | Equality / inequality |
| `gt`, `ge`, `lt`, `le` | Numeric or string range comparison |
| `exists`, `not_exists` | Key presence / absence |
| `in_set`, `not_in_set` | Membership / exclusion |
| `and_*`, `or_*` | Append one predicate with explicit connector |
| `and`, `or`, `and_not`, `or_not` | Append grouped subexpressions |
| `not` | Negate the current expression |

Grouped subexpressions are built with closures. The closure receives a fresh
builder, and the resulting expression is appended as one grouped child:

```rust
use qubit_metadata::{Metadata, MetadataFilter};

let filter = MetadataFilter::builder()
    .eq("status", "active")
    .and(|g| g.ge("score", 80).or_eq("tag", "rust"))
    .build()
    .unwrap();

let meta = Metadata::new()
    .with("status", "active")
    .with("score", 42_i64)
    .with("tag", "rust");

assert!(filter.matches(&meta));
```

The expression above means:

```text
status == "active" AND (score >= 80 OR tag == "rust")
```

Use `and_not` or `or_not` when the whole group should be negated:

```rust
let filter = MetadataFilter::builder()
    .eq("status", "active")
    .and_not(|g| g.ge("score", 80).or_eq("tag", "java"))
    .build()
    .unwrap();
```

Missing-key behavior for negative predicates is controlled by
`MissingKeyPolicy`. Mixed numeric comparison behavior for equality, membership,
and range predicates is controlled by `NumberComparisonPolicy`.

Grouped expressions must contain at least one predicate. For example,
`and(|g| g)` and `or_not(|g| g)` are rejected by `build()` because an empty group
is usually a caller bug.

Empty value sets are allowed. `in_set("key", [])` matches no metadata object.
`not_in_set("key", [])` matches when `key` exists; when `key` is missing, it
follows the configured `MissingKeyPolicy`, just like other negative predicates.

When a schema validates filters, all numeric `DataType` variants are considered
compatible with each other. This intentionally lets callers write convenient
numeric literals in filter conditions even when they cannot precisely mirror the
storage field type. Actual `MetadataSchema::validate(&metadata)` remains strict:
stored metadata values must use the concrete field type declared by the schema.

### 5) Versioned filter serde format

Serialized `MetadataFilter` values use an explicit wire format with `version`,
`expr`, and `options` fields. Expression nodes use `type`, and condition nodes
use stable operator names in `op` such as `eq`, `ge`, `in`, and `not_exists`.
Serialized `Condition` values use the same condition wire representation. The
internal expression tree is not part of the serialization contract. New
serialization always emits the versioned format.

## Error Handling

Use `try_get` and schema validation when the caller needs diagnostics instead of
`Option`:

```rust
use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataError};

let meta = Metadata::new().with("answer", "forty-two");

match meta.try_get::<i64>("answer") {
    Err(MetadataError::TypeMismatch { expected, actual, .. }) => {
        assert_eq!(expected, DataType::Int64);
        assert_eq!(actual, DataType::String);
    }
    other => panic!("unexpected result: {other:?}"),
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
qubit-metadata = "0.4.0"
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
