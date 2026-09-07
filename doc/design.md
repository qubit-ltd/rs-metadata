# qubit-metadata Design

[中文设计文档](design.zh_CN.md) · [User guide](user_guide.md) · [README](../README.md)

This document records the stable design boundaries of `qubit-metadata` 0.10.
It describes invariants for integrations, not implementation history.

## Goals and non-goals

The crate provides an ordered, typed metadata container, optional schema and
filter validation, and strict V1 Serde formats. It is storage-provider neutral.
It does not define a provider indexing strategy, add a new wire version, or
turn diagnostic formatting into a general confidentiality boundary.

## Module boundaries

The core metadata module owns ordered values and typed access. Schema modules
own field declarations and validation. Filter modules own immutable Boolean
expressions, matching options, and receiver-side limits. Wire modules own
versioned Serde envelopes and bounded traversal. JSON budget profiles are
directional: decode and encode policies are configured independently.

## Core data model

`Metadata` maps string keys to `qubit_value::Value` while preserving insertion
order. `Value::Unset` is a present, typed declaration without a concrete value.
`MetadataSchema` distinguishes required and optional fields and validates stored
values against concrete `DataType` declarations. `MetadataFilter` evaluates a
`FilterExpression` against metadata.

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
composition propagates unknown; negation does not turn unknown into true.
Numeric comparison policy is explicit, and approximate comparisons are not
appropriate for ordering or grouping.

## Error taxonomy and redaction

`MetadataError` covers metadata access and schema validation; validation results
can expose multiple independent issues. Filter builders report malformed or
incompatible expressions. Wire errors distinguish domain-limit failures,
syntax/envelope failures, and unsupported versions. Diagnostics preserve useful
types, counts, and versions without embedding rejected metadata values. This is
diagnostic redaction, not a promise to hide arbitrary user keys or error text.

## Feature/dependency architecture

The default feature set is core-only. `filter` supplies filter dependencies,
`schema` includes `filter`, and `json` supplies bounded JSON support and its
JSON budget integration. Value families are independently feature-gated.
Applications should enable only the layers used at each boundary.

## Compatibility policy

Public names and the V1 wire representation remain stable within this release.
The crate keeps strict stored-value schema validation distinct from compatible
numeric filter checks. Integrations should use builders and public Serde APIs,
keep wire data within canonical limits, and treat a future wire-version change
as an explicit compatibility event.
