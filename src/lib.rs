// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # qubit-metadata
//!
//! A general-purpose, typed-value, extensible metadata model for Rust.
//!
//! This crate provides a [`Metadata`] type — a structured key-value store
//! designed for any domain that needs to attach typed annotations to its data
//! models. It is not a plain `HashMap` — it is a structured extensibility point
//! with typed conversion access, [`qubit_value::Value`] backing, and
//! first-class `serde` support.
//!
//! ## Design Goals
//!
//! - **Typed Values**: Conversion-based typed get, chainable set, and
//!   replacement-aware insert APIs backed by [`qubit_value::Value`]
//! - **Generality**: No domain-specific assumptions — usable in any Rust
//!   project
//! - **Schema Support**: Optional schema validation for metadata and filters
//!   through the `schema` feature
//! - **Serialization**: First-class `serde` support for JSON interchange
//! - **Filtering**: Composable query conditions through the `filter` feature
//!
//! ## Features
//!
//! - Core type: [`Metadata`] — an ordered key-value store with typed accessors
//! - Enable `filter` for composable filter expressions and their public types
//! - Enable `schema` (which includes `filter`) for field definitions and
//!   validation APIs
//! - Error type: [`MetadataError`] — explicit failure reporting for `try_*`
//!   APIs
//! - `schema` also provides aggregate validation errors
//!
//! ## Example
//!
//! ```rust
//! use qubit_metadata::Metadata;
//!
//! let meta = Metadata::new()
//!     .with("author", "alice")
//!     .with("priority", 3_i64);
//!
//! // Convenience API: get converts the stored value to the requested target;
//! // missing keys and failed conversions both collapse to None.
//! let author: Option<String> = meta.get("author");
//! assert_eq!(author.as_deref(), Some("alice"));
//!
//! // Explicit API: preserve failure reasons for diagnostics.
//! let priority = meta.try_get::<i64>("priority").unwrap();
//! assert_eq!(priority, 3);
//! ```
//!
//! The `filter` feature uses fail-closed three-valued logic: a missing or unset
//! value stays unknown through negation and does not match at the public API
//! boundary.
//!
//! `Metadata`'s `Debug` and `Display` implementations are bounded and rendered
//! through the default redaction policy. They are useful diagnostic views, not
//! a confidentiality boundary for arbitrary user-defined keys or error text.

#![deny(missing_docs)]

mod constants;
#[cfg(feature = "filter")]
mod filter;
mod metadata;
mod metadata_error;
#[cfg(feature = "json")]
mod metadata_limits;
mod metadata_result;
#[cfg(feature = "schema")]
mod metadata_validation_error;
#[cfg(feature = "schema")]
mod metadata_validation_result;
#[cfg(feature = "json")]
mod metadata_wire_decode_error;
#[cfg(feature = "json")]
mod metadata_wire_encode_error;
mod metadata_wire_limit_kind;
#[cfg(feature = "schema")]
mod schema;
mod wire;

#[cfg(feature = "filter")]
pub use filter::Condition;
#[cfg(feature = "filter")]
pub use filter::FilterExpression;
#[cfg(feature = "filter")]
pub use filter::FilterExpressionBuilder;
#[cfg(feature = "filter")]
pub use filter::FilterExpressionView;
#[cfg(feature = "filter")]
pub use filter::FilterLimitKind;
#[cfg(feature = "filter")]
pub use filter::FilterLimits;
#[cfg(feature = "filter")]
pub use filter::FilterLimitsBuilder;
#[cfg(feature = "filter")]
pub use filter::FilterMatchOptions;
#[cfg(feature = "filter")]
pub use filter::FilterMatchOptionsBuilder;
#[cfg(feature = "filter")]
pub use filter::MetadataFilter;
#[cfg(feature = "filter")]
pub use filter::MetadataFilterBuilder;
pub use metadata::Metadata;
pub use metadata_error::MetadataError;
#[cfg(feature = "json")]
pub use metadata_limits::{
    DEFAULT_MAX_JSON_BYTES,
    DEFAULT_MAX_KEY_BYTES,
    DEFAULT_MAX_METADATA_ENTRIES,
    DEFAULT_MAX_SCHEMA_FIELDS,
    MetadataLimits,
    default_json_decode_limits,
    default_json_encode_limits,
    default_json_value_limits,
};
pub use metadata_result::MetadataResult;
#[cfg(feature = "schema")]
pub use metadata_validation_error::MetadataValidationError;
#[cfg(feature = "schema")]
pub use metadata_validation_result::MetadataValidationResult;
#[cfg(feature = "json")]
pub use metadata_wire_decode_error::MetadataWireDecodeError;
#[cfg(feature = "json")]
pub use metadata_wire_encode_error::MetadataWireEncodeError;
pub use metadata_wire_limit_kind::MetadataWireLimitKind;
#[cfg(feature = "schema")]
pub use schema::MetadataField;
#[cfg(feature = "schema")]
pub use schema::MetadataSchema;
#[cfg(feature = "schema")]
pub use schema::MetadataSchemaBuilder;
#[cfg(feature = "schema")]
pub use schema::UnknownFilterFieldPolicy;
#[cfg(feature = "schema")]
pub use schema::UnknownMetadataFieldPolicy;
