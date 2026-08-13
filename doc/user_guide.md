# qubit-metadata User Guide

[中文用户手册](user_guide.zh_CN.md) · [README](../README.md) · [API documentation](https://docs.rs/qubit-metadata)

This guide targets `qubit-metadata` 0.10 and Rust 1.94 or later. It is for
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
qubit-metadata = "0.10"
qubit-datatype = "0.11"
```

Enable optional layers explicitly when they are used:

```toml
[dependencies]
qubit-metadata = { version = "0.10", features = ["schema", "json"] }
qubit-datatype = "0.11"
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
qubit-metadata = { version = "0.10", default-features = false }
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
assert_eq!(metadata.get_str(TENANT_ID), Some("acme"));
```

When a key/value contract is shared with a storage provider, add a
`MetadataSchema` and validate metadata before it crosses that boundary. The
crate does not normalize key spelling or naming style for callers.

### 2. Read values with the right failure model

Use `get` when both a missing key and a conversion failure should be treated as
absence. Use `get_raw` to inspect the stored `Value` without conversion. Use
`try_get` when diagnostics are part of the application behavior.

```rust
use qubit_metadata::{Metadata, MetadataError};

let metadata = Metadata::new().with("chunk_index", 3_i64);

let index: Option<i64> = metadata.get("chunk_index");
assert_eq!(index, Some(3));

match metadata.try_get::<String>("chunk_index") {
    Err(MetadataError::TypeMismatch { .. }) => {}
    other => panic!("unexpected result: {other:?}"),
}
```

`Value::Unset` is different from a missing key: it remains present and retains
its declared type, but it does not contain a concrete value. `try_get` reports
`MissingKey` for an absent key and `MissingValue` for an unset value.

### 3. Define a schema at the storage boundary

Use `MetadataSchema::builder()` when a backend requires a known field layout or
when records must be validated before insertion. Required fields must exist and
contain a concrete value; optional fields may be omitted. Stored values must
match the concrete `DataType` declared by the field.

```rust
use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataSchema};

let schema = MetadataSchema::builder()
    .required("tenant_id", DataType::String)
    .required("chunk_index", DataType::Int64)
    .optional("language", DataType::String)
    .build()
    .expect("schema should be valid");

let metadata = Metadata::new()
    .with("tenant_id", "acme")
    .with("chunk_index", 3_i64)
    .with("language", "en");

schema.validate(&metadata).expect("metadata should match schema");
```

Unknown metadata fields and unknown filter fields have independent policies.
The default policy rejects unknown names. Allowing an undeclared metadata key
does not automatically allow an unchecked filter key.

### 4. Build and evaluate a filter

`FilterExpression::builder()` describes Boolean structure. The resulting
expression is bound to matching options and limits by `MetadataFilter::builder()`.
Use `build_checked(&schema)` when filter fields and operand types should be
validated before the filter is sent to a backend.

```rust
use qubit_datatype::DataType;
use qubit_metadata::{
    FilterExpression,
    Metadata,
    MetadataFilter,
    MetadataSchema,
};

let schema = MetadataSchema::builder()
    .required("status", DataType::String)
    .required("score", DataType::Int64)
    .build()
    .unwrap();

let expression = FilterExpression::builder()
    .eq("status", "ready")
    .ge("score", 80_i64)
    .build()
    .unwrap();
let filter = MetadataFilter::builder()
    .expression(expression)
    .build_checked(&schema)
    .unwrap();

let metadata = Metadata::new()
    .with("status", "ready")
    .with("score", 92_i64);
assert!(filter.matches(&metadata));
```

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

## Advanced usage

### Three-valued filter semantics

Missing keys and `Value::Unset` evaluate to unknown. The public `matches`
method returns `true` only for a definite true result. Negation preserves
unknown, and AND/OR propagate it. Consequently, `ne("key", value)` and
`not(eq("key", value))` do not match an absent or unset key.

Empty sets are valid: `in_set("key", [])` matches nothing, while
`not_in_set("key", [])` matches only concrete values. Empty groups are rejected
by `build()` because they usually indicate an incomplete query.

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

```rust
use qubit_metadata::{Metadata, MetadataLimits};

let limits = MetadataLimits::default()
    .with_max_metadata_entries(128)
    .with_max_key_bytes(128);
let metadata = Metadata::decode_json_slice_with_limits(
    br#"{"version":1,"values":{"tenant_id":{"scalar":{"string":"acme"}}}}"#,
    limits,
)?;
# Ok::<(), qubit_metadata::MetadataWireDecodeError>(())
```

The default limits are 1,048,576 input bytes, 4,096 metadata entries, 4,096
schema fields, and 256 UTF-8 bytes per key. `MetadataLimits` keeps these
metadata-domain limits separate from its `JsonDecodeLimits` and
`JsonEncodeLimits` profiles; use `with_json_decode` or `with_json_encode` to
replace the relevant generic traversal and directional byte limits. Domain limits cannot
exceed the canonical V1 serialization limits. The input byte limit is checked
before JSON parsing.

`MetadataSchema` and `MetadataFilter` provide corresponding bounded JSON
decoders. Filter decoding additionally accepts receiver-controlled
`FilterLimits`. The explicit filter JSON decoder enforces receiver AST limits
and the shared JSON budget while reading the expression tree. Filter limits are
transient receiver-side policy and are not serialized. Individual JSON strings
and embedded value payloads may still require temporary allocations bounded by
the outer input-byte limit. Generic `serde::Deserialize` remains intended for
an already-bounded outer protocol.

### Strict V1 wire formats

Metadata, schema, and filters serialize through strict V1 envelopes. Unknown
fields, malformed nodes, and unsupported versions are rejected. Filter
envelopes contain only `version`, `expression`, and `options`; a legacy V1
`limits` field is rejected as an unknown field. Expression nodes use tags such
as `eq`, `ge`, `in`, `and`, `or`, `not`, `all`, and `none`.
Do not hand-author these structures unless the versioned format is part of your
integration contract; prefer the public Serde implementations.

`Metadata` and `MetadataSchema` are unbounded in memory. Their serializers
reject maps larger than 4,096 entries or keys longer than 256 UTF-8 bytes only
at the V1 wire boundary, returning a readable serializer error. These hard wire
bounds are shared with the default strict decoders; keep them in producer-side
validation when generating interchange data. Receiver-controlled JSON limits
may be lowered for a particular boundary.

## Errors and diagnostics

Use the error type that matches the boundary being validated:

| Boundary | API | Typical information |
| --- | --- | --- |
| One metadata read | `try_get` | missing key, unset value, conversion/type mismatch |
| One schema check | `MetadataSchema::validate` | all independent metadata issues through `issues()` |
| Filter construction | `build` / `build_checked` | empty groups, invalid operands, unknown fields, incompatible operators |
| JSON input | `decode_json_slice_with_limits` | budget, invalid JSON, or V1 validation failure |

Conversion diagnostics keep structured expected and actual types without
embedding the rejected source value. This makes errors suitable for logs while
avoiding accidental disclosure of metadata contents.

## Troubleshooting

### A filter unexpectedly does not match

Check whether the key is absent or stores `Value::Unset`; both are unknown, not
false, and therefore do not match. Check the declared `DataType` and the
numeric comparison policy. For nested filters, inspect `expression().view()` to
verify the grouping and connector structure.

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

### `get` hides the reason for failure

That is its contract. Replace it with `try_get` and match on `MetadataError` if
the caller must distinguish absence from a type mismatch or unset value.

## Limitations and best practices

- Keep metadata keys and values small enough for the receiving storage system;
  this crate does not provide a storage-provider indexing strategy.
- Validate at the boundary where metadata enters a trusted domain, and apply
  explicit JSON and domain limits before parsing untrusted JSON.
- Use `MetadataFilter::build_checked` when a schema is available. Use the
  unchecked builder only when the target backend owns field validation.
- Treat diagnostic formatting as bounded and redacted output, not a complete
  security policy for arbitrary user-defined keys or error messages.
- Keep serialized metadata and schema maps within the 4,096-entry and
  256-byte-key wire bounds enforced by the Serde implementations.
- Keep the V1 Serde representation behind an integration boundary so a future
  wire-version change can be handled deliberately.

## Next steps

- Read the [README](../README.md) for the project overview and installation
  summary.
- Browse the [API documentation](https://docs.rs/qubit-metadata) for the full
  public surface.
- Run `cargo test --all-features` before changing feature-gated behavior.
- See the [Chinese user guide](user_guide.zh_CN.md) for the same workflow in
  Simplified Chinese.
