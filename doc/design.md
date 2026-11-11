# qubit-metadata Design

[中文设计文档](design.zh_CN.md) · [User guide](user_guide.md) · [README](../README.md)

This document records the stable design boundaries of `qubit-metadata` 0.7.
It describes invariants for integrations, not implementation history.

## Goals and non-goals

The crate provides a key-sorted, typed metadata container, optional schema and
filter validation, and strict V1 Serde formats. It is storage-provider neutral.
It does not define a provider indexing strategy, add a new wire version, or
turn diagnostic formatting into a general confidentiality boundary.

## Module boundaries

The core metadata module owns key-sorted values and typed access. Schema modules
own field declarations and validation. Filter modules own immutable Boolean
expressions, matching options, and receiver-side limits. Wire modules own
versioned Serde envelopes and bounded traversal. JSON budget profiles are
directional: decode and encode policies are configured independently.

With the `json` feature, bounded JSON encoding is available for metadata and,
with `schema`, schemas as well. Enabling both `json` and `filter` also provides
bounded encoding for filters; this does not change the fact that `schema`
includes `filter`. Encoders check value representability and the configured
resource profile before producing compact JSON output. Vector APIs return that
buffer; writer APIs buffer the complete document before writing, but a
destination I/O error can still leave partial bytes at the destination.
Receiver-side filter limits are transient and are not serialized. Applications
own persistence, transaction, transport, retry, and compensation semantics. A
custom `MetadataLimits` encode profile is used only when the caller passes
`limits.json_encode()` to the encoding method; decode settings do not alter
output budgets.

## Core data model

`Metadata` maps string keys to `qubit_value::Value`; iteration and serialization
are sorted by key. `Value::Unset` is a present, typed declaration without a concrete value.
`MetadataSchema` distinguishes required and optional fields and validates stored
values against concrete `DataType` declarations. `MetadataFilter` evaluates a
`FilterExpression` against metadata. Receiver-side limits are runtime policy:
they are excluded from serialization, equality, and hashing.

The `get`, `get_ref`, `get_optional`, and `get_or` family reads strictly using
the value layer's type contract. All return `MetadataResult`; optional reads
hide absence, not type mismatches. Borrowed reads do not clone payloads.
`convert`, `convert_with`, `convert_optional_with`, and `convert_or_with`
explicitly request conversion, with the latter methods accepting policy and
limits. Default values reuse `IntoValueDefault`. Rust evaluates argument expressions before the method call; the adapter is invoked only when fallback is permitted and needed. For example, `metadata.get_or::<String>("tenant", expensive_default())` calls `expensive_default()` before the call whether or not fallback is needed. When passing a borrowed string, only its adaptation to an owned `String` is deferred until fallback is permitted and needed. To defer custom computation, put it in an `IntoValueDefault` adapter; passing a `FnOnce` closure directly is not implied.
The old `try_get*` aliases and `get_str` are removed.

Metadata owns typed application attributes, schema validation, and filter
matching. Configuration source composition, scopes, interpolation, and business
struct deserialization belong to `rs-config`. In particular, `Config::get`
converts while `Metadata::get` requires the exact stored type.

## V1 wire contract and versioning

Metadata, schema, and filter envelopes use strict V1 representations. Unknown
fields, malformed nodes, and unsupported versions are rejected. Filter limits
are receiver-side transient policy and are not serialized. Bounded decoders
report unsupported versions as `UnsupportedVersion`; generic Serde decoding
of schema envelopes rejects positional sequences and requires the canonical
four-field object. Existing object output and the V1 version remain unchanged;
generic Serde decoding retains its ordinary deserializer error channel.
Each value in the schema `fields` map must itself be an object containing
exactly `data_type` and `required`; positional field arrays are rejected.
Consumers with stored V1 schemas that use positional field arrays must read
them with an older library version verified to accept that representation and
re-serialize them as canonical objects before upgrading. The new decoder does
not provide an array fallback.
Filter `options` likewise requires an object containing exactly
`numeric_comparison_policy`; positional arrays are rejected by both the V1
envelope decoder and direct `FilterMatchOptions` deserialization. No in-repo
tests, documentation, or searched downstream callers rely on positional
options. Callers storing such arrays must read them with an older library
version verified to accept that representation and re-serialize them as the canonical object
`{"numeric_comparison_policy":"exact"}` (or the corresponding policy) before
upgrading. Serialized V1 output and the wire version do not change.

## Resource and trust boundaries

Complete untrusted JSON input should use slice decoders with `MetadataLimits`.
The input byte budget is checked before parsing; a shared JSON session then
accounts for generic structure and payload while domain seeds account for
metadata, schema, or filter resources. The default profile includes byte,
depth, node, sequence/map, string/key/number/payload, and domain limits.
In-memory metadata is not intrinsically bounded; V1 wire maps enforce their
canonical entry and key limits.

### Dedicated metadata JSON helpers

The dedicated metadata slice decoder first materializes a unique-key JSON
source tree under the shared JSON budgets, then applies the wire seed and
converts values into the metadata domain representation. This path preserves
native number events when `serde_json/arbitrary_precision` is enabled. Rejected
charges are not applied; accepted input-byte charges remain on failure, while
source-value charges commit after the wire seed succeeds and before later
domain conversion. Domain deserialization errors may no longer retain original
source coordinates. The current unique-key decoder has no additional fixed
127-container limit; the configured generic JSON depth budget still applies.
These are implementation details, not API or performance guarantees. Generic
Serde seeds continue to depend on the events supplied by their outer
deserializer.

The dedicated filter JSON decoder first scans and materializes the complete
unique-key JSON source tree under the shared JSON decode budgets. Its domain
seed then checks receiver AST and membership limits while constructing the
filter. Tightening `FilterLimits` does not stop source-tree materialization;
configure both the generic JSON budgets and domain limits. This materialization
contract applies to the dedicated JSON decoder, not to arbitrary
`serde::Deserialize` deserializers.

## Three-valued filter semantics

Equality, range, and membership comparisons against missing keys or unset
values evaluate to unknown. Incomparable concrete values and failed numeric
comparisons also evaluate to unknown; values are not implicitly converted
between types, and compatibility between numeric types follows
`NumericComparisonPolicy`. Negation preserves unknown, so `ne`, negated
comparisons, and `not_in_set` cannot turn an unknown result into a match.
Existence predicates return definite Booleans: `exists` is false and
`not_exists` is true when no concrete value is stored, so `exists(...).not()`
is true. `matches` returns true only for definite true. Boolean composition
uses dominance (`false AND unknown` is false, `true OR unknown` is true); the
other mixed cases remain unknown. Membership retains its per-candidate
comparison and definite-result precedence.
Numeric comparison policy is explicit, and approximate comparisons are not
appropriate for ordering or grouping.

## Error taxonomy and redaction

Aggregate schema validation retains every issue in `MetadataValidationError::issues()`.
Its `Debug` and `Display` views use one cumulative diagnostic budget; exhaustion
produces an explicit `<redaction incomplete>` marker rather than formatting the
entire collection. The summary still reports the total issue count. Diagnostic
text is not a stable serialization format or a confidentiality boundary for
arbitrary keys and application-supplied messages. Collection itself remains
complete and is not capped by the formatting budget.


`MetadataError` covers metadata access and schema validation; validation results
can expose multiple independent issues. Filter builders report malformed or
incompatible expressions. Wire errors distinguish domain-limit failures,
syntax/envelope failures, and unsupported versions. Bounded metadata and schema
encoding preserve `MetadataError::WireLimitExceeded` in
`MetadataWireEncodeError::Domain`, with schema wire preflight preceding JSON
output budgets; ordinary Serde serialization failures remain in `Json`. Callers
that previously matched `Json` for schema wire limits should match `Domain` and
inspect `kind`, `value`, and `maximum`. Diagnostics preserve useful
types, counts, and versions without embedding rejected metadata values. This is
diagnostic redaction, not a promise to hide arbitrary user keys or error text.
Composite expression, filter, and schema diagnostics share one cumulative
resource budget; output that exceeds it uses an incomplete marker.

`MetadataWireDecodeError::Decode` preserves shared decoder failures for empty input,
invalid UTF-8, unexpected top-level types, and deserialization, including their
`kind()` and structured accessors. `Read`, `Budget`, `Quantity`, and `Syntax` keep
their existing dedicated mappings. Standard metadata entry points use `Redacted`
diagnostics; a `Detailed` source is retained only when the caller selected that
policy before conversion. `InvalidJson` remains manually constructible and is
also returned by the standard metadata slice decoder when a materialized wire
payload cannot be converted to a scalar metadata value, for example when an
entry contains a collection payload. This conversion happens after wire
materialization and may no longer retain original source coordinates. Callers
that matched it as the complete set of JSON failures should
add a `Decode` arm. Display text and source-chain depth are not compatibility
contracts.

`MetadataError::ValueAccess` stores a boxed `ValueError`. Missing reads retain
`ValueMissing` facts: reason, source and target types, optional collection index,
and the original conversion error. `Error::source` preserves that chain.
`MetadataError::MissingValue` is removed; schema's `TypeMismatch` remains a
separate domain error. Optional/default reads use the appropriate value missing
predicate instead of swallowing every error.

## Feature/dependency architecture

The default feature set is core-only. `filter` supplies filter dependencies,
`schema` includes `filter`, and `json` supplies bounded JSON support and its
JSON budget integration. Value families are independently feature-gated.
Applications should enable only the layers used at each boundary.

## Compatibility policy

Compatibility has three separate dimensions: the V1 wire version and canonical
object output, the set of input representations accepted by decoders, and the
error/API contracts exposed to callers. Stable wire version and canonical
output do not guarantee an unchanged input-acceptance range. A registry-backed
consumer using `qubit-metadata 0.6.2` and this 0.7.0 checkout read the same
fixtures in `tests/fixtures/compatibility/`: 0.6.2 accepts positional schema
field arrays and positional filter `options`, but rejects positional schema
envelopes; current decoders reject all three positional forms. The same run
observed error-class changes for invalid UTF-8 and wrong top-level input, and
for bounded entry/key encode limits. Details and migration steps are in the
[0.7 migration checklist](migration_0.7.md#compatibility-checks-before-upgrading).
This verifies only 0.6.2 → this 0.7.0 checkout. It does not determine behavior
for every 0.6.x release or which release first changed acceptance or error
classification. Rust API changes remain a separate compatibility dimension.
