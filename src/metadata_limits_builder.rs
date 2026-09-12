// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Builder for [`crate::MetadataLimits`].

use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonEncodeLimits;

use crate::metadata_limits::DEFAULT_MAX_KEY_BYTES;
use crate::metadata_limits::DEFAULT_MAX_METADATA_ENTRIES;
use crate::metadata_limits::DEFAULT_MAX_SCHEMA_FIELDS;
use crate::metadata_limits::MetadataLimits;
use crate::metadata_limits::default_json_decode_limits;
use crate::metadata_limits::default_json_encode_limits;

/// Builder for [`MetadataLimits`].
///
/// # Examples
///
/// ```
/// use qubit_metadata::MetadataLimits;
///
/// # fn main() -> Result<(), serde_json::Error> {
/// let limits = MetadataLimits::builder().max_metadata_entries(128).build()?;
/// assert_eq!(limits.max_metadata_entries(), 128);
/// # Ok(())
/// # }
/// ```
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetadataLimitsBuilder {
    /// Receiver-controlled JSON budgets applied while decoding untrusted input.
    pub(crate) json_decode: JsonDecodeLimits,
    /// Preflight and output budgets applied while encoding metadata documents.
    pub(crate) json_encode: JsonEncodeLimits,
    /// Cardinality ceiling shared by metadata wire-map operations.
    pub(crate) max_metadata_entries: usize,
    /// Cardinality ceiling shared by schema wire-map operations.
    pub(crate) max_schema_fields: usize,
    /// UTF-8 byte ceiling shared by metadata and schema keys.
    pub(crate) max_key_bytes: usize,
}

impl MetadataLimitsBuilder {
    /// Replaces the JSON decoding profile.
    #[inline(always)]
    #[must_use = "the configured builder must be used to build metadata limits"]
    pub fn json_decode(mut self, limits: JsonDecodeLimits) -> Self {
        self.json_decode = limits;
        self
    }

    /// Replaces the JSON encoding profile.
    #[inline(always)]
    #[must_use = "the configured builder must be used to build metadata limits"]
    pub fn json_encode(mut self, limits: JsonEncodeLimits) -> Self {
        self.json_encode = limits;
        self
    }

    /// Sets the metadata-entry domain limit.
    #[inline(always)]
    #[must_use = "the configured builder must be used to build metadata limits"]
    pub const fn max_metadata_entries(mut self, maximum: usize) -> Self {
        self.max_metadata_entries = maximum;
        self
    }

    /// Sets the schema-field domain limit.
    #[inline(always)]
    #[must_use = "the configured builder must be used to build metadata limits"]
    pub const fn max_schema_fields(mut self, maximum: usize) -> Self {
        self.max_schema_fields = maximum;
        self
    }

    /// Sets the metadata/schema key-byte domain limit.
    #[inline(always)]
    #[must_use = "the configured builder must be used to build metadata limits"]
    pub const fn max_key_bytes(mut self, maximum: usize) -> Self {
        self.max_key_bytes = maximum;
        self
    }

    /// Builds validated metadata limits by consuming this builder.
    ///
    /// # Errors
    ///
    /// Returns a configuration error when a domain limit exceeds its protocol
    /// hard cap, matching [`crate::FilterLimitsBuilder::build`] validation
    /// timing.
    #[inline]
    #[must_use = "the metadata limit validation result must be handled"]
    pub fn build(self) -> Result<MetadataLimits, serde_json::Error> {
        let limits = MetadataLimits::from_builder(self);
        limits.validate()?;
        Ok(limits)
    }
}

impl Default for MetadataLimitsBuilder {
    fn default() -> Self {
        Self {
            json_decode: default_json_decode_limits(),
            json_encode: default_json_encode_limits(),
            max_metadata_entries: DEFAULT_MAX_METADATA_ENTRIES,
            max_schema_fields: DEFAULT_MAX_SCHEMA_FIELDS,
            max_key_bytes: DEFAULT_MAX_KEY_BYTES,
        }
    }
}
