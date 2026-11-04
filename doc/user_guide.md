# qubit-metadata User Guide

[中文用户手册](user_guide.zh_CN.md) · [Migrating to 0.6](migration_0.6.md) · [README](../README.md) · [API documentation](https://docs.rs/qubit-metadata)

This guide targets `qubit-metadata` 0.7 and Rust 1.94 or later. It is for
Rust developers who need to attach typed, queryable metadata to records,
messages, or document chunks without coupling the metadata model to a storage
provider.

## The problem and the model

`Metadata` is an ordered map from string keys to `qubit_value::Value`. The map
is extensible, but values retain their concrete types. A caller can therefore
use a convenient typed read when absence is acceptable, or a diagnostic read
when a missing key and a failed conversion must be distinguished.

The optional layers build on that core:

```text
Metadata -> MetadataSchema -> validation of stored values
        -> FilterExpression -> MetadataFilter -> matching
        -> strict V1 Serde/JSON wire formats
```

`schema` enables schema validation and includes `filter`. The `filter` feature
can be enabled independently when schema validation is not needed. The `json`
feature enables bounded JSON-slice decoding.

## Scenario: indexing document chunks

Assume a pipeline receives a document, splits it into chunks, and sends each
chunk to a storage backend. A successful implementation should be able to:

1. attach source and tenant fields while preserving scalar types;
2. reject incomplete records before storage;
3. construct a filter that can be translated or evaluated locally; and
4. diagnose malformed metadata or untrusted serialized input.

The following sections implement this path incrementally.

## Installation and feature selection

For the core metadata API (the crate's default feature set is core-only):

```toml
[dependencies]
qubit-metadata = "0.7"
```

Enable optional layers explicitly when they are used:

```toml
[dependencies]
qubit-metadata = { version = "0.7", features = ["schema", "json"] }
qubit-datatype = "0.15"
```

The `schema` feature includes `filter`; use `features = ["filter"]` when
schema validation is not needed. There are no default features, so a
metadata-only dependency needs no additional configuration.

The crate declares `filter`, `schema`, `chrono`, `big-integer`, `big-decimal`,
`big-number`, `url`, `json`, and `all`. `all` enables the declared optional
value families and JSON support.

For an explicit metadata-only declaration, disable default features:

```toml
[dependencies]
qubit-metadata = { version = "0.7", default-features = false }
```

This is equivalent to the core-only default today and documents the intended
feature boundary for applications that manage feature flags centrally.

## Core workflow

### 1. Store typed fields

`with` consumes and returns the metadata value, which is convenient for a
single construction expression. `set` mutates an existing value and returns a
mutable reference for chaining. `insert` returns the old value when a key is
replaced.

```rust
use qubit_metadata::Metadata;

let mut metadata = Metadata::new()
    .with("tenant_id", "acme")
    .with("document_id", "doc-42")
    .with("chunk_index", 3_i64)
    .with("language", "en");

metadata.set("indexed", true);
let previous = metadata.insert("chunk_index", 4_i64);
assert!(previous.is_some());
```

Keys are ordered for iteration and serialization. `merge` and `merged` use the
right-hand metadata value when both objects contain the same key.

### 1a. Keep cross-component keys stable

Metadata keys are plain strings, so a spelling difference silently creates a
different field. Define a constant at the boundary that owns a key and reuse it
for every write, read, and filter:

```rust
use qubit_metadata::Metadata;

const TENANT_ID: &str = "tenant_id";

let metadata = Metadata::new().with(TENANT_ID, "acme");
assert_eq!(metadata.get_ref::<str>(TENANT_ID).unwrap(), "acme");
```

When a key/value contract is shared with a storage provider, add a
`MetadataSchema` and validate metadata before it crosses that boundary. The
crate does not normalize key spelling or naming style for callers.

### 2. Read values with the right failure model

Use `get` for a strict typed `Result<T>`, `get_ref` to borrow a typed payload,
and `get_optional` for `Result<Option<T>>`. Optional reads preserve type errors.
Use `get_raw` to inspect the stored `Value`, and `convert` when conversion is
intended. Unlike `Config::get`, `Metadata::get` never implicitly converts.

```rust
use qubit_metadata::{Metadata, MetadataError};

let metadata = Metadata::new().with("chunk_index", 3_i64);

let index: Option<i64> = metadata.get_optional("chunk_index").unwrap();
assert_eq!(index, Some(3));

match metadata.get::<String>("chunk_index") {
    Err(MetadataError::ValueAccess { .. }) => {}
    other => panic!("unexpected result: {other:?}"),
}
```

`Value::Unset` is different from a missing key: it remains present and retains
its declared type, but it does not contain a concrete value. `get` reports
`MissingKey` for an absent key. An appropriately typed unset produces
`ValueAccess` containing `ValueError::Missing`; a mismatched declared type is
still a type error. Optional/default reads absorb only eligible missing states.

### 3. Validate and index a chunk

The schema and query below use the same chunk fields: `tenant_id`,
`chunk_index`, and optional `language`. Validate each record before calling the
application's store. `build_checked(&schema)` separately checks query fields
and operand types before a query is used. A successful local check does not
replace authorization or tenant isolation in the storage layer.

<!-- chunk-workflow-example:start -->
```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_datatype::DataType;
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataSchema;

pub trait ChunkStore {
    fn insert(&mut self, metadata: Metadata) -> Result<(), Box<dyn std::error::Error>>;
}

fn ingest(
    store: &mut impl ChunkStore,
    schema: &MetadataSchema,
    metadata: Metadata,
) -> Result<(), Box<dyn std::error::Error>> {
    schema.validate(&metadata)?;
    store.insert(metadata)
}

#[derive(Default)]
struct RecordingStore(Vec<Metadata>);

impl ChunkStore for RecordingStore {
    fn insert(&mut self, metadata: Metadata) -> Result<(), Box<dyn std::error::Error>> {
        self.0.push(metadata);
        Ok(())
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = MetadataSchema::builder()
        .required("tenant_id", DataType::String)
        .required("chunk_index", DataType::Int64)
        .optional("language", DataType::String)
        .build()?;
    let record = Metadata::new()
        .with("tenant_id", "acme")
        .with("chunk_index", 3_i64)
        .with("language", "en");
    let mut store = RecordingStore::default();
    ingest(&mut store, &schema, record.clone())?;
    assert_eq!(store.0.len(), 1);
    assert!(ingest(&mut store, &schema, Metadata::new().with("tenant_id", "acme")).is_err());
    assert_eq!(store.0.len(), 1);

    let filter = MetadataFilter::builder()
        .expression(
            FilterExpression::builder()
                .eq("tenant_id", "acme")
                .ge("chunk_index", 2_i64)
                .build()?,
        )
        .build_checked(&schema)?;
    assert!(filter.matches(&record));
    assert!(!filter.matches(&Metadata::new().with("tenant_id", "other").with("chunk_index", 3_i64)));

    let invalid_query = FilterExpression::builder().ge("chunk_index", "two").build()?;
    assert!(
        MetadataFilter::builder()
            .expression(invalid_query)
            .build_checked(&schema)
            .is_err()
    );
    Ok(())
}
```
<!-- chunk-workflow-example:end -->

The valid record is accepted, while the record missing `chunk_index` is
rejected before `insert`; the store count stays at one. The checked query
matches the same tenant and does not match another tenant. A query with a
string range operand for the integer field is rejected during
`build_checked`. `RecordingStore` is only a test stand-in that makes these
paths runnable. Implement `ChunkStore` with your persistence layer, including
its transaction, retry, and query translation behavior.

### 4. Group predicates for a separate query

Predicates added without a group connector are joined with AND. Use
`and_group` and `or_group` for nested expressions:

```rust
use qubit_metadata::{FilterExpression, Metadata, MetadataFilter};

let expression = FilterExpression::builder()
    .eq("status", "ready")
    .and_group(|group| {
        group
            .ge("score", 80_i64)
            .or_group(|alternative| alternative.eq("tag", "rust"))
    })
    .build()
    .unwrap();
let filter = MetadataFilter::builder()
    .expression(expression)
    .build()
    .unwrap();

let metadata = Metadata::new()
    .with("status", "ready")
    .with("score", 42_i64)
    .with("tag", "rust");
assert!(filter.matches(&metadata));
```

The expression is `status == "ready" AND (score >= 80 OR tag == "rust")`.
Other builder predicates are `ne`, `gt`, `lt`, `le`, `exists`, `not_exists`,
`in_set`, `not_in_set`, and `not`.

## Migrating from earlier APIs

If you upgrade from releases before 0.6, or still have call sites that relied on
implicit conversion, optional `get`, or `MetadataError::MissingValue`, continue
with [Migrating to 0.6: explicit reads and boundary diagnostics](migration_0.6.md)
after the core workflow above. New projects can skip that note and stay in the
sections below.

## Advanced usage

### Three-valued filter semantics

Equality, range, and membership comparisons against a missing key or
`Value::Unset` produce unknown. Comparisons of concrete values can also produce
unknown when their types or operations are incompatible, or when a numeric
comparison fails. Values are not implicitly converted between types; compatible
numeric types are determined by `NumericComparisonPolicy`. Negation preserves
unknown, so `ne`, negated comparisons, and `not_in_set` cannot bypass an
incomparable type. For a membership condition, candidates are compared
individually and a definite result takes precedence over an unknown candidate.
The public `matches` method returns `true` only for a definite true result.
Existence predicates remain definite: `exists` is false and `not_exists` is
true when no concrete value is stored; consequently, `exists("key").not()` is
also true. Boolean composition uses dominance: `false AND unknown` is `false`,
`true OR unknown` is `true`, and the remaining mixed cases stay unknown.

```rust
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;

let metadata = Metadata::new().with("age", "not-a-number");
let expressions = [
    FilterExpression::builder().ne("age", 18_i64).build().unwrap(),
    FilterExpression::builder().eq("age", 18_i64).not().build().unwrap(),
    FilterExpression::builder().not_in_set("age", [18_i64, 21_i64]).build().unwrap(),
];
for expression in expressions {
    let filter = MetadataFilter::builder().expression(expression).build().unwrap();
    assert!(!filter.matches(&metadata));
}
```

Empty sets are valid. Give an empty iterator an element type, because an empty
array does not provide enough information for `in_set` or `not_in_set` to infer
the value type. This example requires the `filter` feature:

<!-- empty-set-example:start -->
```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_datatype::DataType;
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
use qubit_value::Value;

pub fn main() -> qubit_metadata::MetadataResult<()> {
    let included = MetadataFilter::builder()
        .expression(
            FilterExpression::builder()
                .in_set("key", std::iter::empty::<i64>())
                .build()?,
        )
        .build()?;
    let excluded = MetadataFilter::builder()
        .expression(
            FilterExpression::builder()
                .not_in_set("key", std::iter::empty::<i64>())
                .build()?,
        )
        .build()?;
    let concrete = Metadata::new().with("key", 1_i64);
    assert!(!included.matches(&concrete));
    assert!(excluded.matches(&concrete));
    assert!(!excluded.matches(&Metadata::new()));
    assert!(!excluded.matches(&Metadata::new().with("key", Value::Unset(DataType::Int64))));
    Ok(())
}
```
<!-- empty-set-example:end -->

Empty groups are rejected by `build()` because they usually indicate an
incomplete query.

Numeric comparisons use the configured `NumericComparisonPolicy`. Schema
filter validation accepts non-NaN numeric representations as compatible with a
numeric field, while stored metadata schema validation remains strict about the
declared concrete type. Approximate projected comparisons are pair-dependent
and non-transitive; do not use them for sorting, grouping, `Ord`, or ordered
keys.

### Inspect or translate an expression

`MetadataFilter::expression()` returns the root expression and
`FilterExpression::view()` exposes a read-only tree containing condition, AND,
OR, NOT, true, and false nodes. A storage provider can walk this view and
translate supported predicates into its native query language without relying
on the private construction representation.

### Bounded JSON decoding

When `json` is enabled, use the slice decoding methods for complete untrusted
JSON input:

<!-- bounded-json-decoding-example:start -->
```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;
use qubit_metadata::MetadataWireDecodeError;

pub fn main() -> Result<(), MetadataWireDecodeError> {
    let limits = MetadataLimits::builder()
        .max_metadata_entries(128)
        .max_key_bytes(128)
        .build()
        .map_err(MetadataWireDecodeError::InvalidLimits)?;
    let metadata = Metadata::decode_json_slice_with_limits(
        br#"{"version":1,"values":{"tenant_id":{"scalar":{"string":"acme"}}}}"#,
        limits,
    )?;
    assert_eq!(
        metadata.get_ref::<str>("tenant_id").expect("tenant_id is a string"),
        "acme"
    );
    Ok(())
}
```
<!-- bounded-json-decoding-example:end -->

The default limits are 1,048,576 input bytes, 4,096 metadata entries, 4,096
schema fields, and 256 UTF-8 bytes per key. `MetadataLimits` keeps these
metadata-domain limits separate from its `JsonDecodeLimits` and
`JsonEncodeLimits` profiles; use the `json_decode` or `json_encode` methods on
`MetadataLimitsBuilder` to replace the relevant generic traversal and directional byte limits. Domain limits cannot
exceed the canonical V1 serialization limits. The input byte limit is checked
before JSON parsing.

`MetadataSchema` and `MetadataFilter` provide corresponding bounded JSON
decoders. Filter decoding additionally accepts receiver-controlled
`FilterLimits`. The explicit filter JSON decoder first materializes the
complete unique-key JSON source tree under the shared JSON decode budgets. Its
domain seed then checks receiver AST and membership limits while constructing
the filter. Tightening `FilterLimits` does not stop the preceding source-tree
materialization, so configure both the generic JSON budgets and domain limits.
Filter limits are transient receiver-side policy and are not serialized. Individual JSON strings
and embedded value payloads may still require temporary allocations bounded by
the outer input-byte limit. Generic `serde::Deserialize` remains intended for
an already-bounded outer protocol.

When limits come from an operator or configuration file, call
`MetadataLimits::builder().build()?`. It rejects domain caps above the protocol
hard limits during configuration, matching `FilterLimitsBuilder::build`.

### Bounded JSON encoding

For an application that stores or sends document chunks, encode each chunk at
the application boundary and pass the resulting bytes to its own persistence or
transport code. This complete minimal program configures an output budget,
encodes metadata, decodes it again, and checks that a writer produces the same
wire bytes:

```toml
[dependencies]
qubit-metadata = { version = "0.7", features = ["json"] }
qubit-budget = { version = "0.10", features = ["json"] }
```

<!-- bounded-json-encoding-example:start -->
```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;
use qubit_metadata::default_json_encode_limits;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let encode = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 128 * 1024))
        .build();
    let limits = MetadataLimits::builder().json_encode(encode).build()?;
    let metadata = Metadata::new()
        .with("tenant_id", "acme")
        .with("document_id", "doc-42")
        .with("chunk_index", 3_i64);

    let bytes = metadata.to_json_vec_with_limits(limits.json_encode())?;
    // The application passes these bytes to its own storage or transport layer.
    let restored = Metadata::decode_json_slice_with_limits(&bytes, limits)?;
    assert_eq!(restored, metadata);

    let mut output = Vec::new();
    metadata.to_json_writer_with_limits(&mut output, limits.json_encode())?;
    assert_eq!(output, bytes);
    Ok(())
}
```
<!-- bounded-json-encoding-example:end -->

This is an entry-level exercise, not a storage implementation. The application
still owns transactions, sending, retries, and compensation for I/O failures.
`to_json_vec` and `to_json_writer` use the default output budget; when a
`MetadataLimits` value contains a custom encode profile, pass
`limits.json_encode()` explicitly. Changing `json_decode` does not change that
output profile. `MetadataSchema` has the same `to_json_vec*` and
`to_json_writer*` family when the `schema` feature is enabled. Both metadata and schema bounded encoders return `Domain(MetadataError::WireLimitExceeded { kind, value, maximum })` for V1 wire-limit violations, before checking JSON output budgets. Callers that previously matched `Json` for schema limits should match `Domain` and inspect those fields; ordinary Serde serialization failures remain `Json`.

The encoder checks value representability and budgets before producing the
compact JSON buffer. The writer API then writes that buffered output; a writer
failure can still leave some bytes at the destination. This does not make file
or network writes transactional. A one-byte budget gives a predictable
rejection:

```rust
use qubit_budget::json::JsonResource;
use qubit_budget::ResourceLimit;
use qubit_metadata::default_json_encode_limits;
use qubit_metadata::{Metadata, MetadataWireEncodeError};

let limits = default_json_encode_limits()
    .into_builder()
    .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
    .build();
let metadata = Metadata::new().with("tenant_id", "acme");
assert!(matches!(
    metadata.to_json_vec_with_limits(limits),
    Err(MetadataWireEncodeError::Budget(_))
));
```

Handle the budget failure by reducing the output or changing the configured
budget according to application policy; do not retry the same payload with an
unbounded profile. `MetadataWireEncodeError` also distinguishes `Quantity`, `Syntax`, `Json`, and
`Io`. Metadata and schema wire-contract failures use
`Domain(MetadataError::WireLimitExceeded { kind, value, maximum })`, preserving
the limit category and measurements. Migrate code that previously treated
schema wire-limit failures as `Json` by matching `Domain` and inspecting those
fields; ordinary Serde serialization failures remain `Json`. The enum is `non_exhaustive`, so callers should keep
a fallback arm when matching all variants.

When both `json` and `filter` are enabled, `MetadataFilter` also has bounded JSON
encoding. `to_json_vec` and `to_json_writer` use the default JSON output budget;
`to_json_vec_with_limits` and `to_json_writer_with_limits` accept an explicit
`JsonEncodeLimits`. These limits govern encoded output. Receiver-side
`FilterLimits` are transient policy and are not serialized. The encoder checks
the complete document and its budget before writing to a writer, so a budget
failure writes no bytes; a destination I/O failure can still leave a prefix.

```toml
[dependencies]
qubit-metadata = { version = "0.7", features = ["json", "filter"] }
```

This example builds a tenant filter, encodes and decodes it, checks the match,
and verifies that the writer and Vec APIs produce identical bytes:

<!-- bounded-filter-json-encoding-example:start -->
```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
use qubit_metadata::default_json_encode_limits;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let expression = FilterExpression::builder().eq("tenant", "acme").build()?;
    let filter = MetadataFilter::builder().expression(expression).build()?;
    let bytes = filter.to_json_vec_with_limits(default_json_encode_limits())?;
    let restored = MetadataFilter::decode_json_slice(&bytes)?;
    assert!(restored.matches(&Metadata::new().with("tenant", "acme")));
    let mut output = Vec::new();
    filter.to_json_writer_with_limits(&mut output, default_json_encode_limits())?;
    assert_eq!(output, bytes);
    Ok(())
}
```
<!-- bounded-filter-json-encoding-example:end -->

### Strict V1 wire formats

Metadata, schema, and filters serialize through strict V1 envelopes. Unknown
fields, malformed nodes, and unsupported versions are rejected. Filter
envelopes contain only `version`, `expression`, and `options`; a legacy V1
`limits` field is rejected as an unknown field. Expression nodes use tags such
as `eq`, `ge`, `in`, `and`, `or`, `not`, `all`, and `none`.
Do not hand-author these structures unless the versioned format is part of your
integration contract; prefer the public Serde implementations.

Each value in a schema's `fields` map must be an object containing exactly
`data_type` and `required`; positional field arrays are rejected. If stored V1
schemas use positional field arrays, read them with the previous library
version and re-serialize them as canonical objects before upgrading. The new
decoder has no array fallback.

`Metadata` and `MetadataSchema` are unbounded in memory. Their serializers
reject maps larger than 4,096 entries or keys longer than 256 UTF-8 bytes only
at the V1 wire boundary, returning `MetadataWireEncodeError::Domain` with the
structured `MetadataError::WireLimitExceeded` details for metadata and schema encoding. Schema
encoding checks these domain limits before JSON output budgets. These hard wire bounds are shared
with the default strict decoders; keep them in producer-side
validation when generating interchange data. Receiver-controlled JSON limits
may be lowered for a particular boundary.

## Errors and diagnostics

Use the error type that matches the boundary being validated:

| Boundary | API | Typical information |
| --- | --- | --- |
| One metadata read | `get` / `convert` | missing key, unset value, conversion/type mismatch |
| One schema check | `MetadataSchema::validate` | all independent metadata issues through `issues()` |
| Filter construction | `build` / `build_checked` | empty groups, invalid operands, unknown fields, incompatible operators |
| JSON input | `decode_json_slice_with_limits` | budget, invalid JSON, or V1 validation failure |

Conversion diagnostics keep structured expected and actual types without
embedding the rejected source value. This makes errors suitable for logs while
avoiding accidental disclosure of metadata contents.

For bounded JSON decoding, `Read`, `Budget`, `Quantity`, and `Syntax` remain
separate error categories. Shared decoder failures for empty input, invalid UTF-8,
an unexpected top-level type, or deserialization are reported as
`MetadataWireDecodeError::Decode`; inspect its `kind()` and other accessors for
structured details instead of parsing display text. The standard metadata
entry points use `Redacted` diagnostics. A `Detailed` source is retained only
when the caller explicitly chose that policy before conversion. Existing code
that manually constructs `InvalidJson` can continue to do so, while matches
that assumed it covered all JSON failures should add a `Decode` arm. Error text
and source-chain depth are not compatibility guarantees.

## Troubleshooting

### A filter unexpectedly does not match

First identify the predicate category. Equality, range, and membership
comparisons against an absent or unset value produce unknown, so they do not
match; existence predicates return definite results (`exists` is false and
`not_exists` is true when no concrete value is stored). Then check the declared
`DataType` and numeric comparison policy. For nested filters, inspect
`expression().view()` to verify the grouping and connector structure.

### Schema validation rejects a value that compares numerically

Filter compatibility and stored metadata validation have different purposes.
The former accepts compatible numeric representations for query construction;
the latter requires the stored value to use the field's declared concrete type.
Convert the value before storing it or declare the schema with the intended
type.

### JSON decoding fails before parsing appears to start

Compare the input byte length with the `JsonDecodeLimits` profile held by
`MetadataLimits`. The input-size check is intentionally performed before the
JSON parser is invoked. If the input is within that bound, inspect domain and
generic budget facts in the returned `MetadataWireDecodeError`.

### A getter no longer returns `Option`

Version 0.6 makes `get` strict and fallible. Use `get_optional` when absence is
acceptable and propagate its `Result`; use `convert_optional_with` when you
need policy-controlled conversion. Invalid values are errors, not `None`. See
the [0.6 migration guide](migration_0.6.md) for the full API mapping table.

## Limitations and best practices

- Keep metadata keys and values small enough for the receiving storage system;
  this crate does not provide a storage-provider indexing strategy.
- Validate at the boundary where metadata enters a trusted domain, and apply
  explicit JSON and domain limits before parsing untrusted JSON.
- Use `MetadataFilterBuilder::build_checked` when a schema is available. Use the
  unchecked builder only when the target backend owns field validation.
- Treat diagnostic formatting as bounded and redacted output, not a complete
  security policy for arbitrary user-defined keys or error messages.
- Keep serialized metadata and schema maps within the 4,096-entry and
  256-byte-key wire bounds enforced by the Serde implementations.
- Keep the V1 Serde representation behind an integration boundary so a future
  wire-version change can be handled deliberately.

## Next steps

- Read the [README](../README.md) for the project overview and installation summary.
- Browse the [API documentation](https://docs.rs/qubit-metadata) for the full public surface.
- Read [Migrating to 0.6](migration_0.6.md) when upgrading older call sites.
- Read the [design document](design.md) for module and compatibility invariants.
- Run `cargo test --all-features` before changing feature-gated behavior.
- See the [Chinese user guide](user_guide.zh_CN.md) for the same workflow in Simplified Chinese.
