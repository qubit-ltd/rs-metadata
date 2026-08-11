// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Default JSON and metadata-domain limits for metadata wire documents.

use qubit_budget::{
    JsonDecodeLimits,
    JsonEncodeLimits,
    JsonResource,
    JsonValueLimits,
    ResourceLimit,
    StructureLimits,
};
use serde::de::Error as _;

/// Default maximum complete metadata JSON input or output length.
pub const DEFAULT_MAX_JSON_BYTES: u64 = 1_048_576;

/// Default maximum metadata map entries accepted by the JSON profile.
pub const DEFAULT_MAX_METADATA_ENTRIES: u64 = 4_096;

/// Default maximum schema fields accepted by the JSON profile.
pub const DEFAULT_MAX_SCHEMA_FIELDS: u64 = 4_096;

/// Default maximum UTF-8 bytes in one metadata key.
pub const DEFAULT_MAX_KEY_BYTES: u64 = 256;

/// Domain-specific metadata limits composed with a shared JSON profile.
///
/// `MetadataLimits` deliberately keeps metadata-entry, schema-field, and key
/// bounds separate from generic JSON map/key accounting. This preserves the
/// protocol's domain limits while the directional JSON limit types handle
/// document traversal, payload lengths, and complete input/output bytes.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetadataLimits {
    json_decode: JsonDecodeLimits,
    json_encode: JsonEncodeLimits,
    max_metadata_entries: u64,
    max_schema_fields: u64,
    max_key_bytes: u64,
}

impl MetadataLimits {
    /// Creates metadata limits with the default JSON profile and the supplied
    /// input/output byte bound.
    pub fn new(max_json_bytes: u64) -> Self {
        Self {
            json_decode: default_json_decode_limits().with_input_bytes_limit(
                ResourceLimit::new(JsonResource::InputBytes, max_json_bytes),
            ),
            json_encode: default_json_encode_limits().with_output_bytes_limit(
                ResourceLimit::new(JsonResource::OutputBytes, max_json_bytes),
            ),
            max_metadata_entries: DEFAULT_MAX_METADATA_ENTRIES,
            max_schema_fields: DEFAULT_MAX_SCHEMA_FIELDS,
            max_key_bytes: DEFAULT_MAX_KEY_BYTES,
        }
    }

    /// Replaces the JSON decoding profile.
    pub fn with_json_decode(mut self, limits: JsonDecodeLimits) -> Self {
        self.json_decode = limits;
        self
    }

    /// Replaces the JSON encoding profile.
    pub fn with_json_encode(mut self, limits: JsonEncodeLimits) -> Self {
        self.json_encode = limits;
        self
    }

    /// Returns the JSON decoding profile.
    pub const fn json_decode(&self) -> JsonDecodeLimits {
        self.json_decode
    }

    /// Returns the JSON encoding profile.
    pub const fn json_encode(&self) -> JsonEncodeLimits {
        self.json_encode
    }

    /// Sets the metadata-entry domain limit.
    pub const fn with_max_metadata_entries(mut self, maximum: u64) -> Self {
        self.max_metadata_entries = maximum;
        self
    }

    /// Sets the schema-field domain limit.
    pub const fn with_max_schema_fields(mut self, maximum: u64) -> Self {
        self.max_schema_fields = maximum;
        self
    }

    /// Sets the metadata/schema key-byte domain limit.
    pub const fn with_max_key_bytes(mut self, maximum: u64) -> Self {
        self.max_key_bytes = maximum;
        self
    }

    /// Returns the metadata-entry domain limit.
    pub const fn max_metadata_entries(&self) -> u64 {
        self.max_metadata_entries
    }

    /// Returns the schema-field domain limit.
    pub const fn max_schema_fields(&self) -> u64 {
        self.max_schema_fields
    }

    /// Returns the metadata/schema key-byte domain limit.
    pub const fn max_key_bytes(&self) -> u64 {
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
        Self::new(DEFAULT_MAX_JSON_BYTES)
    }
}

/// Creates the default direction-independent metadata JSON value profile.
pub fn default_json_value_limits() -> JsonValueLimits {
    JsonValueLimits::default()
        .with_string_bytes_limit(ResourceLimit::new(
            JsonResource::StringBytes,
            256 * 1024,
        ))
        .with_number_bytes_limit(ResourceLimit::new(
            JsonResource::NumberBytes,
            4_096,
        ))
        .with_payload_bytes_limit(ResourceLimit::new(
            JsonResource::PayloadBytes,
            DEFAULT_MAX_JSON_BYTES,
        ))
        .with_structure_limits(
            StructureLimits::empty()
                .with_depth_limit(ResourceLimit::new(JsonResource::Depth, 64))
                .with_nodes_limit(ResourceLimit::new(
                    JsonResource::Nodes,
                    100_000,
                ))
                .with_sequence_items_limit(ResourceLimit::new(
                    JsonResource::SequenceItems,
                    4_096,
                ))
                .with_map_entries_limit(ResourceLimit::new(
                    JsonResource::MapEntries,
                    DEFAULT_MAX_METADATA_ENTRIES,
                ))
                .with_key_bytes_limit(ResourceLimit::new(
                    JsonResource::KeyBytes,
                    DEFAULT_MAX_KEY_BYTES,
                )),
        )
}

/// Creates the default metadata JSON decoding profile.
pub fn default_json_decode_limits() -> JsonDecodeLimits {
    JsonDecodeLimits::default()
        .with_input_bytes_limit(ResourceLimit::new(
            JsonResource::InputBytes,
            DEFAULT_MAX_JSON_BYTES,
        ))
        .with_value_limits(default_json_value_limits())
}

/// Creates the default metadata JSON encoding profile.
pub fn default_json_encode_limits() -> JsonEncodeLimits {
    JsonEncodeLimits::default()
        .with_output_bytes_limit(ResourceLimit::new(
            JsonResource::OutputBytes,
            DEFAULT_MAX_JSON_BYTES,
        ))
        .with_value_limits(default_json_value_limits())
}
