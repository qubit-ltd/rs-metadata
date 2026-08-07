// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Resource limits for untrusted metadata JSON wire input.

pub use crate::constants::{
    DEFAULT_MAX_JSON_BYTES,
    DEFAULT_MAX_KEY_BYTES,
    DEFAULT_MAX_METADATA_ENTRIES,
    DEFAULT_MAX_SCHEMA_FIELDS,
};
use crate::{
    MetadataWireDecodeError,
    MetadataWireLimitKind,
};
use qubit_value::WireLimits;

/// Immutable resource limits applied to JSON wire decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct MetadataWireLimits {
    /// Shared Value and JSON structural limits.
    wire: WireLimits,
    /// Maximum number of input bytes accepted before parsing begins.
    max_json_bytes: usize,
    /// Maximum entries in a decoded metadata object.
    max_metadata_entries: usize,
    /// Maximum field definitions in a decoded metadata schema.
    max_schema_fields: usize,
    /// Maximum UTF-8 byte length of one decoded key.
    max_key_bytes: usize,
}

impl MetadataWireLimits {
    /// Creates JSON wire limits with default decoded-resource bounds.
    ///
    /// # Parameters
    ///
    /// * `max_json_bytes` - Largest accepted JSON input length in bytes.
    ///
    /// # Returns
    ///
    /// Limits that reject inputs longer than `max_json_bytes` and apply the
    /// default decoded-resource bounds.
    #[inline(always)]
    pub const fn new(max_json_bytes: usize) -> Self {
        Self {
            wire: WireLimits::new(max_json_bytes),
            max_json_bytes,
            max_metadata_entries: DEFAULT_MAX_METADATA_ENTRIES,
            max_schema_fields: DEFAULT_MAX_SCHEMA_FIELDS,
            max_key_bytes: DEFAULT_MAX_KEY_BYTES,
        }
    }

    /// Sets the shared Value and JSON structural limits.
    #[inline(always)]
    #[must_use = "the configured wire limits should be used"]
    pub const fn with_wire(mut self, wire: WireLimits) -> Self {
        self.wire = wire;
        self.max_json_bytes = wire.max_input_bytes();
        self
    }

    /// Returns the shared Value and JSON structural limits.
    #[inline(always)]
    pub const fn wire(&self) -> WireLimits {
        self.wire
    }

    /// Sets the largest accepted decoded metadata entry count.
    ///
    /// # Parameters
    ///
    /// * `max_metadata_entries` - Maximum entries accepted in one object.
    ///
    /// # Returns
    ///
    /// This limit with the requested metadata-entry bound.
    #[inline(always)]
    #[must_use = "the configured metadata-entry limit should be used"]
    pub const fn with_max_metadata_entries(
        mut self,
        max_metadata_entries: usize,
    ) -> Self {
        self.max_metadata_entries = max_metadata_entries;
        self
    }

    /// Sets the largest accepted decoded metadata-schema field count.
    ///
    /// # Parameters
    ///
    /// * `max_schema_fields` - Maximum fields accepted in one schema.
    ///
    /// # Returns
    ///
    /// This limit with the requested schema-field bound.
    #[inline(always)]
    #[must_use = "the configured schema-field limit should be used"]
    pub const fn with_max_schema_fields(
        mut self,
        max_schema_fields: usize,
    ) -> Self {
        self.max_schema_fields = max_schema_fields;
        self
    }

    /// Sets the largest accepted decoded metadata or schema key length.
    ///
    /// # Parameters
    ///
    /// * `max_key_bytes` - Maximum UTF-8 key length in bytes.
    ///
    /// # Returns
    ///
    /// This limit with the requested key-byte bound.
    #[inline(always)]
    #[must_use = "the configured key-byte limit should be used"]
    pub const fn with_max_key_bytes(mut self, max_key_bytes: usize) -> Self {
        self.max_key_bytes = max_key_bytes;
        self
    }

    /// Returns the maximum accepted JSON input length.
    ///
    /// # Returns
    ///
    /// The number of bytes accepted before JSON parsing begins.
    #[inline(always)]
    pub const fn max_json_bytes(&self) -> usize {
        self.max_json_bytes
    }

    /// Returns the maximum accepted decoded metadata entry count.
    #[inline(always)]
    #[must_use]
    pub const fn max_metadata_entries(&self) -> usize {
        self.max_metadata_entries
    }

    /// Returns the maximum accepted decoded schema field count.
    #[inline(always)]
    #[must_use]
    pub const fn max_schema_fields(&self) -> usize {
        self.max_schema_fields
    }

    /// Returns the maximum accepted decoded metadata or schema key length.
    #[inline(always)]
    #[must_use]
    pub const fn max_key_bytes(&self) -> usize {
        self.max_key_bytes
    }

    /// Rejects `input_bytes` when it exceeds this limit.
    ///
    /// # Parameters
    ///
    /// * `input_bytes` - Complete byte length of the untrusted input.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataWireDecodeError::InputTooLarge`] before a parser is
    /// invoked when `input_bytes` exceeds this limit.
    #[inline(always)]
    pub fn check_json_bytes(
        &self,
        input_bytes: usize,
    ) -> Result<(), MetadataWireDecodeError> {
        if input_bytes > self.max_json_bytes {
            return Err(MetadataWireDecodeError::InputTooLarge {
                input_bytes,
                max_input_bytes: self.max_json_bytes,
            });
        }
        Ok(())
    }

    /// Checks decoded metadata entry count against this limit.
    pub(crate) fn check_metadata_entries(
        &self,
        entries: usize,
    ) -> Result<(), MetadataWireDecodeError> {
        self.check_limit(
            MetadataWireLimitKind::MetadataEntries,
            entries,
            self.max_metadata_entries,
        )
    }

    /// Checks decoded metadata-schema field count against this limit.
    #[cfg(feature = "schema")]
    pub(crate) fn check_schema_fields(
        &self,
        fields: usize,
    ) -> Result<(), MetadataWireDecodeError> {
        self.check_limit(
            MetadataWireLimitKind::SchemaFields,
            fields,
            self.max_schema_fields,
        )
    }

    /// Checks a decoded metadata or schema key length against this limit.
    pub(crate) fn check_key_bytes(
        &self,
        key_bytes: usize,
    ) -> Result<(), MetadataWireDecodeError> {
        self.check_limit(
            MetadataWireLimitKind::KeyBytes,
            key_bytes,
            self.max_key_bytes,
        )
    }

    /// Checks one decoded resource against its configured limit.
    fn check_limit(
        &self,
        kind: MetadataWireLimitKind,
        value: usize,
        maximum: usize,
    ) -> Result<(), MetadataWireDecodeError> {
        if value > maximum {
            return Err(MetadataWireDecodeError::LimitExceeded {
                kind,
                value,
                maximum,
            });
        }
        Ok(())
    }
}

impl Default for MetadataWireLimits {
    /// Creates the standard one-mebibyte JSON input limit.
    #[inline(always)]
    fn default() -> Self {
        Self::new(DEFAULT_MAX_JSON_BYTES)
    }
}
