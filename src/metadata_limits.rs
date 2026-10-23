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

/// Default maximum number of bytes in a complete metadata JSON document.
///
/// This bound applies to both the complete input accepted during decoding and
/// the complete output produced during encoding. Directional JSON profiles
/// apply it through their respective input and output byte budgets.
pub const DEFAULT_MAX_JSON_BYTES: usize = 1_048_576;

/// Default maximum number of entries accepted in a metadata map.
///
/// This domain limit is separate from the generic JSON map-entry budget.
pub const DEFAULT_MAX_METADATA_ENTRIES: usize = 4_096;

/// Default maximum number of fields accepted in a metadata schema.
///
/// This domain limit applies to schema fields independently of generic JSON
/// traversal limits.
pub const DEFAULT_MAX_SCHEMA_FIELDS: usize = 4_096;

/// Default maximum UTF-8 byte length of one metadata or schema key.
///
/// The limit counts encoded bytes, so a multi-byte character consumes more
/// than one byte.
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
/// # fn main() -> Result<(), serde_json::Error> {
/// let limits = MetadataLimits::builder().max_key_bytes(128).build()?;
/// assert_eq!(limits.max_key_bytes(), 128);
/// # Ok(())
/// # }
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
    ///
    /// # Returns
    ///
    /// A builder initialized with the protocol defaults.
    #[inline(always)]
    #[must_use = "the builder must be configured or used to build metadata limits"]
    pub fn builder() -> MetadataLimitsBuilder {
        MetadataLimitsBuilder::default()
    }

    /// Constructs domain limits without validating their protocol hard caps.
    ///
    /// This constructor is available only in debug builds and is intended for
    /// tests that need to exercise wire-boundary handling of invalid limits.
    /// The supplied values are preserved as-is.
    ///
    /// # Parameters
    ///
    /// * `max_metadata_entries` - Maximum metadata-map entry count.
    /// * `max_schema_fields` - Maximum schema-field count.
    /// * `max_key_bytes` - Maximum UTF-8 byte length of a metadata or schema
    ///   key.
    ///
    /// # Returns
    ///
    /// A limits value containing the supplied domain limits and the default
    /// JSON profiles.
    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_only_invalid_domain_limits(
        max_metadata_entries: usize,
        max_schema_fields: usize,
        max_key_bytes: usize,
    ) -> Self {
        MetadataLimits::from_builder(
            MetadataLimitsBuilder::default()
                .max_metadata_entries(max_metadata_entries)
                .max_schema_fields(max_schema_fields)
                .max_key_bytes(max_key_bytes),
        )
    }

    /// Creates immutable limits from the values held by `builder` without
    /// validating the domain hard caps.
    ///
    /// # Parameters
    ///
    /// * `builder` - The builder whose JSON profiles and domain limits are
    ///   copied into the resulting value.
    ///
    /// # Returns
    ///
    /// A limits value containing the builder's configured values.
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

    /// Returns the JSON budgets used to decode an untrusted document.
    ///
    /// The returned value is copied and can be used to inspect the profile
    /// without changing this limits value.
    ///
    /// # Returns
    ///
    /// The decoding profile stored in this value.
    #[inline(always)]
    #[must_use]
    pub const fn json_decode(&self) -> JsonDecodeLimits {
        self.json_decode
    }

    /// Returns the JSON budgets used to encode a document.
    ///
    /// The returned value is copied and can be used to inspect the profile
    /// without changing this limits value.
    ///
    /// # Returns
    ///
    /// The encoding profile stored in this value.
    #[inline(always)]
    #[must_use]
    pub const fn json_encode(&self) -> JsonEncodeLimits {
        self.json_encode
    }

    /// Returns the maximum number of entries accepted in a metadata map.
    ///
    /// This domain-specific bound is independent of generic JSON map
    /// accounting.
    ///
    /// # Returns
    ///
    /// The metadata-entry limit stored in this value.
    #[inline(always)]
    #[must_use]
    pub const fn max_metadata_entries(&self) -> usize {
        self.max_metadata_entries
    }

    /// Returns the maximum number of fields accepted in a metadata schema.
    ///
    /// This domain-specific bound is independent of generic JSON traversal
    /// limits.
    ///
    /// # Returns
    ///
    /// The schema-field limit stored in this value.
    #[inline(always)]
    #[must_use]
    pub const fn max_schema_fields(&self) -> usize {
        self.max_schema_fields
    }

    /// Returns the maximum UTF-8 byte length accepted for one key.
    ///
    /// The byte count applies to metadata and schema keys and is distinct from
    /// the generic JSON key budget.
    ///
    /// # Returns
    ///
    /// The key-byte limit stored in this value.
    #[inline(always)]
    #[must_use]
    pub const fn max_key_bytes(&self) -> usize {
        self.max_key_bytes
    }

    /// Validates receiver-controlled domain limits against protocol hard caps.
    ///
    /// # Errors
    ///
    /// Returns a JSON error when the metadata-entry, schema-field, or key-byte
    /// limit exceeds its corresponding protocol maximum.
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
    /// Creates the protocol's default metadata and JSON limits.
    ///
    /// The defaults satisfy the domain hard caps, so construction cannot fail.
    ///
    /// # Returns
    ///
    /// A limits value containing the standard decoding, encoding, and domain
    /// profiles.
    fn default() -> Self {
        Self::builder()
            .build()
            .expect("default metadata limits satisfy protocol hard caps")
    }
}

/// Creates the default direction-independent metadata JSON value profile.
///
/// The profile bounds nesting, nodes, sequence items, map entries, keys,
/// strings, numbers, and payload size. Complete input and output byte limits
/// are supplied separately by the directional profiles.
///
/// # Returns
///
/// A value profile configured with the protocol defaults.
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
///
/// The profile applies the protocol's complete-input byte limit together with
/// the shared metadata JSON value limits.
///
/// # Returns
///
/// A decoding profile configured with the protocol defaults.
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
///
/// The profile applies the protocol's complete-output byte limit together
/// with the shared metadata JSON value limits.
///
/// # Returns
///
/// An encoding profile configured with the protocol defaults.
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
///
/// The protocol and parser currently represent these quantities with the same
/// unsigned integer type, so conversion preserves the input exactly.
///
/// # Parameters
///
/// * `value` - A protocol limit expressed as a `usize`.
///
/// # Returns
///
/// The unchanged quantity in the parser's native representation.
fn json_quantity(value: usize) -> usize {
    value
}
