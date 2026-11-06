# Migrating from 0.6 to 0.7

[中文迁移说明](migration_0.7.zh_CN.md) · [User guide](user_guide.md) · [README](../README.md) · [Design](design.md) · [API documentation](https://docs.rs/qubit-metadata)

This guide covers dependency coordination when moving to `qubit-metadata`
0.7. The comparison below uses the published `qubit-metadata` 0.6.2 manifest
from the local Cargo registry and this repository's 0.7.0 manifest. It does not
claim that every 0.6.x release has the same dependency set.

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
