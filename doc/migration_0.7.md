# Migrating from 0.6 to 0.7

[中文迁移说明](migration_0.7.zh_CN.md) · [User guide](user_guide.md) · [README](../README.md) · [Design](design.md) · [API documentation](https://docs.rs/qubit-metadata)

This guide covers dependency coordination and checks against the current target
contracts when moving to `qubit-metadata` 0.7. The comparison below uses the
published `qubit-metadata` 0.6.2 manifest from the local Cargo registry and this
repository's 0.7.0 manifest. It does not claim that every 0.6.x release has the
same dependency set.

## Compatibility checks before upgrading

Use this checklist when your application stores, sends, or classifies metadata
wire data. These are the current target-version contracts; the first release
that introduced each input or error-contract change has not been confirmed.
The stable design document describes the [V1 wire contract](design.md#v1-wire-contract-and-versioning)
and [error taxonomy](design.md#error-taxonomy-and-redaction).

| Check | Applies when | Current contract and action |
| --- | --- | --- |
| Schema envelope and field objects | You persist or produce schema JSON, especially if it may contain positional sequences | The envelope must be the canonical four-field object, and every `fields` value must be an object containing exactly `data_type` and `required`. Both positional envelope sequences and positional field arrays are rejected. If stored data uses either form, back it up, read it with an older library version verified to accept that representation, encode it as canonical objects, validate it with the target version, then switch consumers. Keep the backup until the target-version validation succeeds. See the [schema wire contract](design.md#v1-wire-contract-and-versioning). |
| Filter `options` | You persist/send filter JSON or deserialize `FilterMatchOptions` directly, and options may be a positional array | `options` must be an object containing exactly `numeric_comparison_policy`; positional arrays are rejected by the envelope decoder and direct options deserialization. Preserve the stored policy when encoding the canonical object; do not replace every policy with `exact`. For legacy arrays, use the same backup → verified old-version read → canonical object encoding → target-version validation → consumer switch sequence. See the [filter options contract](design.md#v1-wire-contract-and-versioning). |
| Decode error classification | Callers treat `InvalidJson` as the complete set of JSON decoding failures | `Decode` carries shared decoder failures such as empty input, invalid UTF-8, unexpected top-level types, and deserialization failures, with structured accessors. Add handling for `Decode` while retaining the dedicated `Read`, `Budget`, `Quantity`, and `Syntax` cases and the existing `InvalidJson` case. See the [decode error contract](design.md#error-taxonomy-and-redaction). |
| Bounded encode wire limits | Callers handle metadata/schema wire-limit failures only through the `Json` encode-error branch | Bounded metadata and schema encoders report wire-limit failures as `Domain(MetadataError::WireLimitExceeded { .. })`; handle that variant and its `kind`, `value`, and `maximum`. Ordinary Serde serialization failures remain in `Json`. See the [bounded encode error contract](design.md#error-taxonomy-and-redaction). |

The positional-array changes are input-acceptance changes; they do not change
canonical V1 output or the wire version. Do not infer that an older release
accepts a stored representation: verify it against the exact library version
and data shape before using it for migration.

## Dependency versions

| Crate | 0.6.2 requirement | 0.7.0 requirement |
| --- | --- | --- |
| `qubit-datatype` | `0.14` | `^0.15` |
| `qubit-value` | `0.12` | `^0.13` |
| `qubit-budget` | `0.7` | `^0.10` |
| `qubit-json` | `0.10` | `^0.12` |
| `qubit-redact` | `0.9` | `^0.10` |

These are the dependency requirements declared by those manifests. The crate
also uses `serde` and optional `qubit-utils`; they are not part of this version
mapping. Cargo may resolve patch releases within each compatible requirement.

## Coordinate dependencies you expose

If your application only calls `qubit-metadata` APIs, update its dependency
and let Cargo resolve the transitive graph. Add a direct dependency on a
Qubit crate when your own code names or constructs that crate's public types.
For example, schema fields use `qubit_datatype::DataType`, so a schema-enabled
application that constructs fields should align its direct datatype
dependency:

```toml
[dependencies]
qubit-metadata = { version = "0.7", features = ["schema", "json"] }
qubit-datatype = "0.15"
```

If your code constructs `qubit_value::Value`, add `qubit-value = "0.13"`.
If it names JSON budget types, add `qubit-budget = { version = "0.10",
features = ["json"] }`. If your application directly names or exposes an error
type from one of these crates, align that crate's direct dependency as well.
Do not add `qubit-json` or `qubit-redact` solely because they appear in the
table; they are implementation dependencies unless your application directly
uses their APIs or error types.

The direct versions matter when a type crosses a function boundary. For
example, a library exposing `Value` in a public signature and its caller must
agree on the same `qubit-value` crate identity. Multiple versions can coexist
inside a Cargo graph, but their similarly named Rust types are distinct and
cannot be passed interchangeably. Upgrade the owner of a shared public
signature and its callers together. If versions must temporarily be isolated,
convert values at an explicit application boundary; Serde is not a general
promise of a lossless bridge between arbitrary crate versions.

## Inspect the resolved graph

After changing `Cargo.toml`, inspect duplicate packages and the resolved
metadata dependency:

```bash
cargo tree -d
cargo tree -i qubit-metadata@0.7.0
```

The inverse query requires the version actually selected by Cargo; replace
`0.7.0` if your lockfile resolves another 0.7 patch release. Review duplicate
`qubit-value`, `qubit-datatype`, or `qubit-budget` entries when those types are
exposed across your own APIs. A duplicate alone is not necessarily an error:
it is a concern when values of the distinct versions need to cross a typed
boundary.

## Keep the 0.6 API migration separate

This guide does not reclassify API changes that were already made in 0.6.
If upgrading from before 0.6, or if old call sites still use the removed
`try_get*`, `get_str`, or `MetadataError::MissingValue` APIs, follow
[Migrating to 0.6](migration_0.6.md) as well. For the 0.7 workflows, feature
selection, and error handling, see the [user guide](user_guide.md).

The version table describes manifest requirements, not a guarantee that every
downstream crate should add all five packages as direct dependencies. Depend
directly on the types your application actually names, then verify the graph
at integration boundaries.
