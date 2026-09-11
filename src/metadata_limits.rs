// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Default JSON and metadata-domain limits for metadata wire documents.

use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;
use serde::de::Error as _;

use crate::metadata_limits_builder::MetadataLimitsBuilder;

/// Default maximum complete metadata JSON input or output length.
pub const DEFAULT_MAX_JSON_BYTES: usize = 1_048_576;

/// Default maximum metadata map entries accepted by the JSON profile.
pub const DEFAULT_MAX_METADATA_ENTRIES: usize = 4_096;

/// Default maximum schema fields accepted by the JSON profile.
pub const DEFAULT_MAX_SCHEMA_FIELDS: usize = 4_096;

/// Default maximum UTF-8 bytes in one metadata key.
pub const DEFAULT_MAX_KEY_BYTES: usize = 256;

/// Domain-specific metadata limits composed with a shared JSON profile.
///
/// `MetadataLimits` deliberately keeps metadata-entry, schema-field, and key
/// bounds separate from generic JSON map/key accounting. This preserves the
/// protocol's domain limits while the directional JSON limit types handle
/// document traversal, payload lengths, and complete input/output bytes.
///
/// # Examples
///
/// ```
/// use qubit_metadata::MetadataLimits;
///
/// let limits = MetadataLimits::builder().max_key_bytes(128).build();
/// assert_eq!(limits.max_key_bytes(), 128);
/// ```
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetadataLimits {
    /// JSON budgets applied while decoding an untrusted document.
    json_decode: JsonDecodeLimits,
    /// JSON budgets applied while encoding a document.
    json_encode: JsonEncodeLimits,
    /// Maximum number of entries accepted in one metadata map.
    max_metadata_entries: usize,
    /// Maximum number of fields accepted in one metadata schema.
    max_schema_fields: usize,
    /// Maximum UTF-8 byte length accepted for a metadata or schema key.
    max_key_bytes: usize,
}

impl MetadataLimits {
    /// Creates a builder with the default JSON profile and domain limits.
    #[inline]
    #[must_use = "the builder must be configured or used to build metadata limits"]
    pub fn builder() -> MetadataLimitsBuilder {
        MetadataLimitsBuilder::default()
    }

    /// Creates immutable limits from the values held by `builder`.
    #[inline]
    pub(crate) fn from_builder(builder: MetadataLimitsBuilder) -> Self {
        Self {
            json_decode: builder.json_decode,
            json_encode: builder.json_encode,
            max_metadata_entries: builder.max_metadata_entries,
            max_schema_fields: builder.max_schema_fields,
            max_key_bytes: builder.max_key_bytes,
        }
    }

    /// Returns the JSON decoding profile.
    #[inline]
    #[must_use]
    pub const fn json_decode(&self) -> JsonDecodeLimits {
        self.json_decode
    }

    /// Returns the JSON encoding profile.
    #[inline]
    #[must_use]
    pub const fn json_encode(&self) -> JsonEncodeLimits {
        self.json_encode
    }

    /// Returns the metadata-entry domain limit.
    #[inline]
    #[must_use]
    pub const fn max_metadata_entries(&self) -> usize {
        self.max_metadata_entries
    }

    /// Returns the schema-field domain limit.
    #[inline]
    #[must_use]
    pub const fn max_schema_fields(&self) -> usize {
        self.max_schema_fields
    }

    /// Returns the metadata/schema key-byte domain limit.
    #[inline]
    #[must_use]
    pub const fn max_key_bytes(&self) -> usize {
        self.max_key_bytes
    }

    /// Validates receiver-controlled domain limits against protocol hard caps.
    pub fn validate(&self) -> Result<(), serde_json::Error> {
        if self.max_metadata_entries > DEFAULT_MAX_METADATA_ENTRIES {
            return Err(serde_json::Error::custom(format!(
                "metadata entries limit {} exceeds {}",
                self.max_metadata_entries, DEFAULT_MAX_METADATA_ENTRIES,
            )));
        }
        if self.max_schema_fields > DEFAULT_MAX_SCHEMA_FIELDS {
            return Err(serde_json::Error::custom(format!(
                "schema fields limit {} exceeds {}",
                self.max_schema_fields, DEFAULT_MAX_SCHEMA_FIELDS,
            )));
        }
        if self.max_key_bytes > DEFAULT_MAX_KEY_BYTES {
            return Err(serde_json::Error::custom(format!(
                "metadata key limit {} exceeds {}",
                self.max_key_bytes, DEFAULT_MAX_KEY_BYTES,
            )));
        }
        Ok(())
    }
}

impl Default for MetadataLimits {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Creates the default direction-independent metadata JSON value profile.
pub fn default_json_value_limits() -> JsonValueLimits {
    JsonValueLimits::<JsonResource, usize>::builder()
        .max_depth(64_usize)
        .max_nodes(100_000_usize)
        .max_sequence_items(4_096_usize)
        .max_map_entries(json_quantity(DEFAULT_MAX_METADATA_ENTRIES))
        .max_key_bytes(256 * 1024_usize)
        .max_string_bytes(256 * 1024_usize)
        .max_number_bytes(4_096_usize)
        .max_payload_bytes(json_quantity(DEFAULT_MAX_JSON_BYTES))
        .build()
}

/// Creates the default metadata JSON decoding profile.
pub fn default_json_decode_limits() -> JsonDecodeLimits {
    JsonDecodeLimits::builder()
        .input_bytes_limit(ResourceLimit::new(
            JsonResource::InputBytes,
            json_quantity(DEFAULT_MAX_JSON_BYTES),
        ))
        .value_limits(default_json_value_limits())
        .build()
}

/// Creates the default metadata JSON encoding profile.
pub fn default_json_encode_limits() -> JsonEncodeLimits {
    JsonEncodeLimits::builder()
        .output_bytes_limit(ResourceLimit::new(
            JsonResource::OutputBytes,
            json_quantity(DEFAULT_MAX_JSON_BYTES),
        ))
        .value_limits(default_json_value_limits())
        .build()
}

/// Converts protocol JSON limits to the native quantity used by JSON parsing.
fn json_quantity(value: usize) -> usize {
    value
}
