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
//! designed for any domain that needs to attach arbitrary, typed, serializable
//! annotations to its data models. It is not a plain `HashMap` — it is a
//! structured extensibility point with type-safe access, `serde_json::Value`
//! backing, and first-class `serde` support.
//!
//! ## Design Goals
//!
//! - **Type Safety**: Typed get/set API backed by `serde_json::Value`
//! - **Generality**: No domain-specific assumptions — usable in any Rust project
//! - **Extensibility**: Acts as a structured extension point, not a stringly-typed bag
//! - **Serialization**: First-class `serde` support for JSON interchange
//! - **Filtering**: [`MetadataFilter`] for composable query conditions
//!
//! ## Features
//!
//! - Core type: [`Metadata`] — an ordered key-value store with typed accessors
//! - Filter type: [`MetadataFilter`] — composable filter expressions for metadata queries
//! - Condition type: [`Condition`] — individual comparison predicates
//! - Error type: [`MetadataError`] — explicit failure reporting for `try_*` APIs
//! - Type inspection: [`MetadataValueType`] — lightweight JSON value type inspection
//!
//! ## Example
//!
//! ```rust
//! use qubit_metadata::{Metadata, MetadataFilter};
//!
//! let mut meta = Metadata::new();
//! meta.set("author", "alice");
//! meta.set("priority", 3_i64);
//!
//! // Convenience API: missing key and type mismatch both collapse to None.
//! let author: Option<String> = meta.get("author");
//! assert_eq!(author.as_deref(), Some("alice"));
//!
//! // Explicit API: preserve failure reasons for diagnostics.
//! let priority = meta.try_get::<i64>("priority").unwrap();
//! assert_eq!(priority, 3);
//!
//! let filter = MetadataFilter::equal("author", "alice")
//!     .unwrap()
//!     .and(MetadataFilter::greater_equal("priority", 1_i64).unwrap());
//! assert!(filter.matches(&meta));
//! ```
//!
//! ## Author
//!
//! Haixing Hu

#![deny(missing_docs)]

mod condition;
mod metadata;
mod metadata_error;
mod metadata_filter;
mod metadata_result;
mod metadata_value_type;

pub use condition::Condition;
pub use metadata::Metadata;
pub use metadata_error::MetadataError;
pub use metadata_filter::{MetadataFilter, MissingKeyPolicy, NumberComparisonPolicy};
pub use metadata_result::MetadataResult;
pub use metadata_value_type::MetadataValueType;
