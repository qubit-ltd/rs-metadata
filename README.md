# Qubit Metadata

[![Rust CI](https://github.com/qubit-ltd/rs-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-metadata.svg?color=blue)](https://crates.io/crates/qubit-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-metadata` is a typed, ordered metadata model for Rust applications that
need extensible fields without weakening the types of their core data model.
It combines conversion-based access, optional schemas, composable filters, and
strict Serde wire formats in one small API.

## A real use case

Suppose a document pipeline stores chunks that may later be sent to a vector
database. The chunk type can keep stable fields in its own struct and use
`Metadata` for fields that vary by source or indexing backend:

```rust
use qubit_metadata::Metadata;

let metadata = Metadata::new()
    .with("tenant_id", "acme")
    .with("document_id", "doc-42")
    .with("chunk_index", 3_i64)
    .with("language", "en");

let tenant: Option<String> = metadata.get("tenant_id");
assert_eq!(tenant.as_deref(), Some("acme"));
assert_eq!(metadata.try_get::<i64>("chunk_index").unwrap(), 3);
```

The stored values retain concrete runtime types through `qubit_value::Value`.
When a backend needs predictable fields and query validation, add a
`MetadataSchema` and build a `MetadataFilter` against it. The complete
scenario, including diagnostics and wire limits, is covered in the
[English user guide](doc/user_guide.md).

## Why this project exists

Plain maps are convenient, but they leave important decisions to every caller:
which values are valid, whether a field is required, how a filter is grouped,
and how untrusted serialized input is bounded. This crate centralizes those
decisions while keeping the core metadata container independent of any storage
provider or domain model.

## Installation

```toml
[dependencies]
qubit-metadata = "0.10"
```

The default feature set provides the core metadata container only. Enable
`schema` when you need schema validation; it includes `filter`:

```toml
[dependencies]
qubit-metadata = { version = "0.10", features = ["schema"] }
```

Optional features are `chrono`, `big-integer`, `big-decimal`, `big-number`,
`url`, `json`, and `all`. Use the direct `qubit-datatype` dependency when
declaring schema field types, and `qubit-value` when constructing `Value`
operands directly.

## What it provides

- `Metadata`: ordered `String -> Value` storage with `get`, diagnostic
  `try_get`, `set`, `insert`, `with`, iteration, merge, and schema-checked
  writes.
- `MetadataSchema`: required/optional field definitions, concrete
  `qubit_datatype::DataType` validation, and independent policies for unknown
  metadata and filter fields.
- `FilterExpression` and `MetadataFilter`: immutable Boolean expressions with
  equality, range, membership, existence, grouping, negation, matching options,
  and receiver-side expression limits.
- Strict V1 Serde formats for metadata, schemas, and filters. The `json`
  feature also provides bounded JSON-slice decoding.
- Structured `MetadataError`, validation errors, and wire-decode errors for
  callers that need to distinguish missing keys, unset values, type mismatches,
  invalid expressions, and input-limit failures.

## Important boundaries

- `get` intentionally collapses a missing key and a failed conversion to
  `None`; use `try_get` when the reason matters.
- `Value::Unset` records a declared type but is not a concrete value. It is
  rejected by required schema fields and does not satisfy filter predicates.
- Filter evaluation is fail-closed three-valued logic: unknown values do not
  become matches through negation.
- Schema validation of stored metadata remains strict about the declared
  concrete field type, even though filter schema checks accept compatible
  numeric representations.
- Default JSON decoding limits input bytes, entry counts, schema field counts,
  key length, and Value wire structure. Configured metadata-entry, schema-field,
  and key limits cannot exceed the canonical V1 serialization limits. Filter
  limits are transient receiver-side policy and are omitted from the V1 wire;
  filter AST and nested Value limits are validated while decoding, so use a
  streaming deserializer or a smaller outer input limit when peak allocation
  must be bounded. Generic Serde deserialization also enforces hard map-entry
  and key-length bounds, but it is not a substitute for caller-controlled
  input-byte and Value-structure limits on untrusted outer protocols.
- Redacted `Debug` and `Display` output is intended for diagnostics, not as a
  confidentiality boundary for arbitrary user keys or error text.
- Metadata keys are plain strings. When a key crosses a module, provider, or
  storage boundary, define one string constant at the owning boundary and use
  it for both writes and reads; use `MetadataSchema` when the key/value contract
  must be validated.

## Learn more

- [English user guide](doc/user_guide.md)
- [中文用户手册](doc/user_guide.zh_CN.md)
- [Rust API documentation](https://docs.rs/qubit-metadata)
- [中文 README](README.zh_CN.md)

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
