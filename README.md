# Qubit Metadata

[![Rust CI](https://github.com/qubit-ltd/rs-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-metadata/coverage/)
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
`Value::Unset` retains its declared type but has no concrete metadata value:
typed reads report `MetadataError::MissingValue`, required schema fields reject
it, and filters treat it like an absent key.

## Design Goals

- **Typed values**: preserve concrete runtime types with `qubit_value::Value`.
- **Convenient construction**: support mutable chaining with `set()`, explicit
  replacement through `insert()`, and owned chaining with `with()`.
- **Optional schema**: validate field names, required fields, concrete `DataType`s, and filter compatibility.
- **Filter builder**: build immutable `MetadataFilter` values with a chainable builder.
- **Serde support**: serialize metadata, schemas, and filters for config, storage, and service boundaries.

## Features

### 1) Typed metadata storage

`Metadata` is an ordered `String -> Value` map. It supports typed `get`,
chainable `set`, replacement-aware `insert`, `try_get`, schema-checked
`set_checked` / `insert_checked` / `with_checked`, fluent `with`, iteration,
merge, retain, and conversion to/from `BTreeMap<String, Value>`.

When keys conflict, `merge` and `merged` give precedence to the right-hand
`other` metadata value.

With the `json` feature, `decode_json_slice` and
`decode_json_slice_with_limits` bound complete untrusted JSON input before
Serde parsing and reject decoded metadata or schema maps that exceed configured
entry, field, or key-length limits. Defaults accept at most 1,048,576 input
bytes, 4,096 metadata entries, 4,096 schema fields, and 256 bytes per key.
Generic Serde deserialization remains appropriate only when an outer protocol
already bounds input. Metadata `Debug` and `Display` are policy-redacted;
ordinary outer keys also recursively redact keyed structures stored in `Value`.

```rust
use qubit_metadata::Metadata;

let meta = Metadata::new()
    .with("author", "alice")
    .with("priority", 3_i64)
    .with("reviewed", true);

assert_eq!(meta.get::<String>("author").as_deref(), Some("alice"));
assert_eq!(meta.try_get::<i64>("priority").unwrap(), 3);
```

Mutable writes can be chained without moving the metadata object. Use
`insert()` instead when the previous value is needed:

```rust
use qubit_metadata::Metadata;

let mut meta = Metadata::new();
meta.set("author", "alice").set("priority", 3_i64);
let previous = meta.insert("priority", 4_i64);
assert!(previous.is_some());
```

`Value::Unset` records a declared type without supplying a concrete value:

| Operation | Absent key | Stored `Unset` | Concrete value |
|---|---|---|---|
| `contains_key` | `false` | `true` | `true` |
| `get_raw` | `None` | `Some(Unset)` | `Some(value)` |
| `data_type` | `None` | Declared type | Concrete type |
| `try_get` | `MissingKey` | `MissingValue` | Value or conversion error |
| Required schema field | Rejected | Rejected | Validated |
| Filter `exists` | `false` | `false` | `true` |
| Comparison filter | No match | No match | Compared normally |

### 2) Schema for validation and storage planning

`MetadataSchema` uses `qubit_datatype::DataType`. This is useful when a storage
backend requires metadata fields to be declared in advance, and it also lets the
filter builder validate field/operator compatibility early. Its
`UnknownMetadataFieldPolicy` and `UnknownFilterFieldPolicy` are independent:
metadata can allow undeclared storage keys without also allowing unchecked
filter keys. Both policies reject unknown keys by default; filter keys are
accepted only with `UnknownFilterFieldPolicy::AllowUnchecked`.

```rust
use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataSchema};

let schema = MetadataSchema::builder()
    .required("tenant_id", DataType::String)
    .required("score", DataType::Int64)
    .optional("source", DataType::String)
    .build()
    .expect("schema should build");

let meta = Metadata::new()
    .with("tenant_id", "acme")
    .with("score", 42_i64);

schema.validate(&meta).unwrap();
```

### 3) Immutable filters built by builder

`FilterExpression::builder()` constructs the required Boolean expression, while
`MetadataFilter::builder()` binds that expression to matching options and
resource limits. Calling `build()` returns a `Result<MetadataFilter,
MetadataError>`. `build_checked(&schema)` also validates referenced fields,
operator compatibility, and filter value types. Schema-level validation returns
an aggregate error so callers can report all independent issues in one pass.

```rust
use qubit_datatype::DataType;
use qubit_metadata::{FilterExpression, Metadata, MetadataFilter, MetadataSchema};

let schema = MetadataSchema::builder()
    .required("status", DataType::String)
    .required("score", DataType::Int64)
    .build()
    .expect("schema should build");

let expression = FilterExpression::builder()
    .eq("status", "active")
    .ge("score", 10)
    .build()
    .unwrap();
let filter = MetadataFilter::builder()
    .expression(expression)
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
| `exists`, `not_exists` | Concrete-value presence / absence |
| `in_set`, `not_in_set` | Membership / exclusion |
| `and_group`, `or_group` | Append grouped subexpressions |
| `not` | Negate the current expression |

Predicates added without a connector are joined by AND. Grouped
subexpressions are built with closures. The closure receives a fresh builder,
and the resulting expression is appended as one grouped child:

```rust
use qubit_metadata::{FilterExpression, Metadata, MetadataFilter};

let expression = FilterExpression::builder()
    .eq("status", "active")
    .and_group(|group| {
        group.ge("score", 80).or_group(|alternative| alternative.eq("tag", "rust"))
    })
    .build()
    .unwrap();
let filter = MetadataFilter::builder()
    .expression(expression)
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

Use `FilterExpression::try_not()` when an already-built expression should be
negated:

```rust
let expression = FilterExpression::builder()
    .eq("status", "active")
    .build()
    .unwrap()
    .try_not()
    .unwrap();
let filter = MetadataFilter::builder()
    .expression(expression)
    .build()
    .unwrap();
```

Missing keys and `Value::Unset` use fail-closed three-valued logic. A comparison
that has no concrete stored value is unknown; `not` preserves unknown, AND/OR
propagate it, and the final `matches()` result is `true` only for a definite
true expression. Consequently, `ne(key, value)` and `not(eq(key, value))` are
equivalent and neither matches a missing or unset value. Mixed numeric
comparison behavior for equality, membership, and range predicates is
controlled by `NumericComparisonPolicy`.

Grouped expressions must contain at least one predicate. For example,
`and_group(|group| group)` is rejected by `build()` because an empty group is
usually a caller bug.

Empty value sets are allowed. `in_set("key", [])` matches no metadata object.
`not_in_set("key", [])` matches only when `key` stores a concrete value; an
absent or unset value remains unknown and therefore does not match.

Comparison and membership operands must be concrete values that the V1 wire
format can represent. In particular, `Value::Unset`, `NaN`, and infinite
floating-point operands are rejected while building or decoding a filter.

Schema validation accepts every non-NaN numeric representation as compatible
with every numeric field; this compatibility check is independent of the
runtime comparison policy. During matching, `Exact` compares represented
mathematical values without rounding. `Approximate` orders primitive infinities
separately; when a finite primitive float participates, it attempts to project
both operands to finite `f64` values and falls back to exact comparison if
either operand cannot be projected that way. Projected comparison is
pair-dependent and non-transitive, so it must not be used for sorting,
grouping, `Ord`, or ordered keys. Actual
`MetadataSchema::validate(&metadata)` remains strict: stored metadata values
must use the concrete field type declared by the schema.

The complete Boolean structure is available through
`MetadataFilter::expression()` and `FilterExpression::view()`. The read-only
view exposes condition, AND, OR, NOT, true, and false nodes without exposing the
private construction representation, so storage providers can translate a
filter into their native query language.

### 5) Versioned filter serde format

Serialized `MetadataFilter` values use the strict v4 wire format with
`version`, `expression`, `options`, and `limits` fields. Every expression node
uses a `kind` tag. Condition data is inline in nodes such as `eq`, `ge`, `in`,
and `not_exists`; Boolean nodes use `and`, `or`, `not`, `all`, and `none`.
Policy enum values use lowercase underscore names such as `exact` and
`approximate`. Unknown fields, malformed nodes, and versions other than v4 are
rejected. `Metadata` and `MetadataSchema` separately use strict v1 envelopes;
their unknown fields and unsupported versions are also rejected.

## Error Handling

Use `try_get` and schema validation when the caller needs diagnostics instead of
`Option`. Single-entry accessors return `MetadataError`; schema-level validation
returns `MetadataValidationError`, whose `issues()` method exposes all collected
`MetadataError` values. Conversion diagnostics report structured reasons
without embedding the rejected source value:

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
qubit-metadata = "0.10"
# Required when naming schema data types or numeric comparison policies.
qubit-datatype = "0.10"
# Required when directly constructing Value operands.
qubit-value = "0.10"
```

### Feature flags

The default feature set preserves the complete metadata, filter, and schema
API. Consumers that need only `Metadata` can disable default features; this
excludes the `filter` and `schema` modules:

```toml
[dependencies]
qubit-metadata = { version = "0.10", default-features = false }
```

Enable rich value types in addition to the default APIs as needed:

```toml
[dependencies]
qubit-metadata = { version = "0.10", features = ["chrono", "json"] }
```

Available features are `filter`, `schema` (which enables `filter`), `chrono`,
`big-integer`, `big-decimal`, `big-number`, `url`, `json`, and `all`.

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-metadata](https://github.com/qubit-ltd/rs-metadata)
