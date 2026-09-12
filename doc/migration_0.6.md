# Migrating to 0.6: explicit reads and boundary diagnostics

[中文迁移说明](migration_0.6.zh_CN.md) · [User guide](user_guide.md) · [Design document](design.md)

This note is for teams upgrading to `qubit-metadata` 0.6 from earlier releases.
It complements the [user guide](user_guide.md) core workflow with API renames,
read semantics, and structured wire diagnostics that changed in 0.6.

## Explicit read APIs

For an indexing pipeline, preserve the stored type when validating an identifier,
and request conversion explicitly when accepting external text:

```rust
use qubit_metadata::Metadata;
let metadata = Metadata::new().with("port", "8080").with("count", 3_i64);
assert_eq!(metadata.get::<i64>("count").unwrap(), 3);
assert!(metadata.get::<i64>("port").is_err());
assert_eq!(metadata.convert::<u16>("port").unwrap(), 8080);
assert_eq!(metadata.get_ref::<str>("port").unwrap(), "8080");
```

`get` uses `StrictValueRead`; `get_ref::<str>` borrows stored text.
`convert_with` accepts `ConversionPolicy` and `ConversionLimits`, with a fresh
budget per call. `convert_optional_with` and `convert_or_with` apply the same
explicit policy to optional/defaulted reads. `get_or` reads strictly and uses
`IntoValueDefault` only when an absent key or appropriately typed unset allows
fallback. Conversion defaults additionally allow scalar policy-missing results;
invalid conversion is never a default.

Unlike `Config::get`, `Metadata::get` never implicitly converts. Version 0.6
makes `get` strict and fallible. Use `get_optional` when absence is acceptable
and propagate its `Result`; use `convert_optional_with` when you need
policy-controlled conversion. Invalid values are errors, not `None`.

## Read and missing-value diagnostics

Read errors retain `ValueError` under `MetadataError::ValueAccess` and expose
it through `Error::source`. A `ValueError::Missing` contains `ValueMissing`
with `reason()`, `source_type()`, `target_type()`, `source_index()`, and the
original conversion error when present. Do not flatten these facts into text.

Schema's `MetadataError::TypeMismatch` remains a schema error. Metadata filter
features and fail-closed matching semantics remain unchanged.

## API and error mapping

| Old call or error | Replacement | Behavior change |
| --- | --- | --- |
| `get::<T>(key)` returning `Option` | `get_optional::<T>(key)` | Returns `Result<Option<T>>`, strictly checks types |
| Converting `try_get::<T>(key)` | `convert::<T>(key)` | Explicit conversion intent |
| `try_get_strict::<T>(key)` | `get::<T>(key)` | Strict reads are the default named getter |
| `get_str` / `try_get_str` | `get_ref::<str>` | Borrowed, fallible strict read |
| `try_convert` / `try_convert_with` | `convert` / `convert_with` | No compatibility aliases |
| Converting `get_or` | `convert_or_with` | Explicit policy and limits; invalid values propagate |
| `MetadataError::MissingValue` | `ValueAccess` containing `ValueError::Missing` | Preserves missing facts and source chain |

## Wire decode diagnostics

Bounded metadata/schema JSON decoding reports domain entry/key limits as
`MetadataWireDecodeError::Domain(MetadataError::WireLimitExceeded { .. })` and
unsupported versions as `UnsupportedVersion { expected, actual }`. These facts
contain counts and version numbers, not metadata values. Syntax and other
strict-envelope failures retain the default redacted JSON diagnostic policy.
The V1 representation has not changed.

## Filter builder fail-fast semantics

Filter builders reject oversized inputs while constructing the expression.
Membership iterators are consumed at most 129 times (128 allowed values plus
one overflow probe); the probe is not converted. After the first failure,
later operands are not converted, membership iterators are not consumed, and
group callbacks are skipped. Rust still evaluates the arguments to each call.
Node/depth checks apply after each composition, including nested groups.

## Downstream integration notes

At downstream boundaries, document each shared key's concrete type, units, and
absence rule. A storage schema adapter must reject constraints it cannot express;
passing a logical schema check alone does not prove backend operator support.
`merge` still overwrites matching keys: batch aggregators must retain per-batch
fields explicitly when request identifiers or other values may conflict.
