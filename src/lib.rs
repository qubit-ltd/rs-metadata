/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # qubit-metadata
//!
//! A general-purpose, type-safe, extensible metadata model for Rust.
//!
//! This crate provides a [`Metadata`] type — a structured key-value store
//! designed for any domain that needs to attach typed annotations to its data
//! models. It is not a plain `HashMap` — it is a structured extensibility point
//! with type-safe access, [`qubit_value::Value`] backing, and first-class
//! `serde` support.
//!
//! ## Design Goals
//!
//! - **Type Safety**: Typed get/set API backed by [`qubit_value::Value`]
//! - **Generality**: No domain-specific assumptions — usable in any Rust project
//! - **Schema Support**: Optional [`MetadataSchema`] validation for metadata and filters
//! - **Serialization**: First-class `serde` support for JSON interchange
//! - **Filtering**: [`MetadataFilter`] for composable query conditions
//!
//! ## Features
//!
//! - Core type: [`Metadata`] — an ordered key-value store with typed accessors
//! - Schema type: [`MetadataSchema`] — field definitions based on [`qubit_common::DataType`]
//! - Filter type: [`MetadataFilter`] — composable filter expressions for metadata queries
//! - Condition type: [`Condition`] — individual comparison predicates
//! - Error type: [`MetadataError`] — explicit failure reporting for `try_*` APIs
//!
//! ## Example
//!
//! ```rust
//! use qubit_metadata::{Metadata, MetadataFilter};
//!
//! let meta = Metadata::new()
//!     .with("author", "alice")
//!     .with("priority", 3_i64);
//!
//! // Convenience API: missing key and type mismatch both collapse to None.
//! let author: Option<String> = meta.get("author");
//! assert_eq!(author.as_deref(), Some("alice"));
//!
//! // Explicit API: preserve failure reasons for diagnostics.
//! let priority = meta.try_get::<i64>("priority").unwrap();
//! assert_eq!(priority, 3);
//!
//! let filter = MetadataFilter::builder()
//!     .eq("author", "alice")
//!     .and_ge("priority", 1_i64)
//!     .build();
//! assert!(filter.matches(&meta));
//! ```
//!
//! ## Author
//!
//! Haixing Hu

#![deny(missing_docs)]

mod condition;
mod filter_match_options;
mod metadata;
mod metadata_field;
mod metadata_error;
mod metadata_filter;
mod metadata_filter_builder;
mod metadata_result;
mod metadata_schema;
mod metadata_schema_builder;
mod missing_key_policy;
mod number_comparison_policy;
mod unknown_field_policy;

pub use condition::Condition;
pub use filter_match_options::FilterMatchOptions;
pub use metadata::Metadata;
pub use metadata_field::MetadataField;
pub use metadata_error::MetadataError;
pub use metadata_filter::MetadataFilter;
pub use metadata_filter_builder::MetadataFilterBuilder;
pub use metadata_result::MetadataResult;
pub use metadata_schema::MetadataSchema;
pub use metadata_schema_builder::MetadataSchemaBuilder;
pub use missing_key_policy::MissingKeyPolicy;
pub use number_comparison_policy::NumberComparisonPolicy;
pub use unknown_field_policy::UnknownFieldPolicy;
