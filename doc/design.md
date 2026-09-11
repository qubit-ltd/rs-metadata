# qubit-metadata Design

[中文设计文档](design.zh_CN.md) · [User guide](user_guide.md) · [README](../README.md)

This document records the stable design boundaries of `qubit-metadata` 0.11.
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
limits. Default parameters reuse `IntoValueDefault`, evaluated only when needed.
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
retains its ordinary deserializer error channel.

## Resource and trust boundaries

Complete untrusted JSON input should use slice decoders with `MetadataLimits`.
The input byte budget is checked before parsing; a shared JSON session then
accounts for generic structure and payload while domain seeds account for
metadata, schema, or filter resources. The default profile includes byte,
depth, node, sequence/map, string/key/number/payload, and domain limits.
In-memory metadata is not intrinsically bounded; V1 wire maps enforce their
canonical entry and key limits.

## Three-valued filter semantics

Predicates evaluate to true, false, or unknown. Missing keys and unset values
are unknown. `matches` returns true only for definite true, and Boolean
composition uses dominance (`false AND unknown` is false, `true OR unknown` is
true); the other mixed cases remain unknown. Negation does not turn unknown
into true.
Numeric comparison policy is explicit, and approximate comparisons are not
appropriate for ordering or grouping.

## Error taxonomy and redaction

`MetadataError` covers metadata access and schema validation; validation results
can expose multiple independent issues. Filter builders report malformed or
incompatible expressions. Wire errors distinguish domain-limit failures,
syntax/envelope failures, and unsupported versions. Diagnostics preserve useful
types, counts, and versions without embedding rejected metadata values. This is
diagnostic redaction, not a promise to hide arbitrary user keys or error text.

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

Version 0.11 changes read signatures and missing error handling as described
above; no compatibility aliases are retained. The V1 wire representation,
filter semantics, and schema validation behavior remain unchanged.
The crate keeps strict stored-value schema validation distinct from compatible
numeric filter checks. Integrations should use builders and public Serde APIs,
keep wire data within canonical limits, and treat a future wire-version change
as an explicit compatibility event.
