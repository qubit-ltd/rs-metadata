# Migrating from 0.6 to 0.7

[中文迁移说明](migration_0.7.zh_CN.md) · [User guide](user_guide.md) · [README](../README.md) · [Design](design.md) · [API documentation](https://docs.rs/qubit-metadata)

This guide covers dependency coordination and checks against the current target
contracts when moving to `qubit-metadata` 0.7. The runtime comparison below was
run between registry `qubit-metadata` 0.6.2 and this repository's 0.7.0
checkout. It does not establish behavior for every 0.6.x release or identify
the first release that changed a contract.

## Compatibility checks before upgrading

Use this checklist when your application stores, sends, or classifies metadata
wire data. The matrix records observed results for the exact 0.6.2-to-0.7.0
comparison; it does not establish when each difference first appeared. The
stable design document describes the [V1 wire contract](design.md#v1-wire-contract-and-versioning)
and [error taxonomy](design.md#error-taxonomy-and-redaction).

| Case / API | Registry 0.6.2 observation | Current 0.7.0 observation | Upgrade impact and migration |
| --- | --- | --- | --- |
| schema_object / MetadataSchema::decode_json_slice | Accepted; canonical object output | Accepted; canonical object output | No input-shape change observed. |
| schema_envelope_sequence / schema decoder | Rejected as InvalidJson | Rejected as structured Decode | No acceptance change; callers may need to account for the error-class change. |
| schema_field_sequence / schema decoder | Accepted; normalized to object fields | Rejected as structured Decode | Back up, read with the exact 0.6.2 consumer, serialize the schema to canonical objects (the observed output preserves the `name` field, `string` type, `required: true`, and both `reject` policies), validate with 0.7, then switch consumers. |
| options_object / direct FilterMatchOptions | Accepted with approximate | Accepted with approximate | No input-shape change observed. |
| options_exact_object / direct FilterMatchOptions | Accepted with exact | Accepted with exact | No input-shape change observed; the probe covers exact and approximate policies. |
| options_sequence / direct FilterMatchOptions | Accepted; normalized to object retaining approximate | Rejected by direct Serde deserialization | Read with 0.6.2, serialize the options object, preserve its policy, and validate in 0.7. Do not rewrite all policies to exact. |
| filter_options_sequence / filter decoder | Accepted; normalized filter retains approximate | Rejected as structured Decode | Back up, read with 0.6.2, serialize the filter with object options, validate in 0.7, then switch consumers. |
| empty / bounded metadata, schema, and filter decoders | Rejected as Syntax | Rejected as Syntax | No classification change observed. |
| invalid_utf8 / bounded metadata, schema, and filter decoders | Rejected as InvalidJson (shared decoder classified invalid UTF-8) | Rejected as structured Decode | Add Decode handling and use its structured kind rather than display text. |
| wrong_top_level / bounded metadata, schema, and filter decoders | Rejected as InvalidJson | Rejected as structured Decode | Same input remains rejected; callers may need to add Decode handling. |
| invalid_syntax / bounded metadata, schema, and filter decoders | Rejected as Syntax | Rejected as Syntax | No classification change observed. |
| 4,097 metadata entries or schema fields / bounded encode | Rejected as Json(CustomSerialization) | Rejected as Domain(MetadataError::WireLimitExceeded { kind: Entries, value: 4097, maximum: 4096 }) | Handle Domain and inspect the structured limit facts; 0.6.2's public encode error did not expose these facts. |
| 257-byte metadata or schema key / bounded encode | Rejected as Json(CustomSerialization) | Rejected as Domain(MetadataError::WireLimitExceeded { kind: KeyBytes, value: 257, maximum: 256 }) | Handle Domain and inspect the structured limit facts; 0.6.2's public encode error did not expose these facts. |

The old consumer writes accepted cases' actual serialized output to
`tests/fixtures/compatibility/normalized/`; current tests decode those same
bytes and compare schema fields, required flags, unknown-field policies, and
numeric policies. The comparison uses the independent consumer in
[tests/fixtures/compatibility-0.6.2/](../tests/fixtures/compatibility-0.6.2/)
and byte-identical shared cases in
[tests/fixtures/compatibility/](../tests/fixtures/compatibility/). Current
assertions are in [compatibility_contract_tests.rs](../tests/compatibility_contract_tests.rs).
The commands below completed with exit code 0 after generating the consumer
lockfile:

```sh
cargo run --locked --manifest-path tests/fixtures/compatibility-0.6.2/Cargo.toml
cargo tree --manifest-path tests/fixtures/compatibility-0.6.2/Cargo.toml -i qubit-metadata@0.6.2
cargo test --all-features --test compatibility_contract_tests -- --nocapture
```

The isolated
consumer enables schema,json; its lockfile resolves registry
qubit-metadata 0.6.2, qubit-datatype 0.14.0, qubit-value 0.12.2, and the 0.6-era
JSON crates. The current test ran against checkout package version 0.7.0 with
all features. The lockfile and source tree are reproducible version and
dependency evidence; the observed release boundary is 0.6.2 to this 0.7.0
checkout.

The verified positional-array input changes do not alter canonical V1 output
or the wire version. The evidence confirms 0.6.2 accepts positional schema
field values and positional filter options, but rejects positional schema
envelopes. It does not establish behavior for other 0.6.x releases or identify
the first release that changed acceptance.

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
