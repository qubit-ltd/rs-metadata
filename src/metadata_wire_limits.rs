// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Byte limits for untrusted metadata JSON wire input.

use crate::MetadataWireDecodeError;

/// Maximum JSON input size used by the default bounded decoding APIs.
pub const DEFAULT_MAX_JSON_BYTES: usize = 1_048_576;

/// Immutable byte limit applied before JSON wire decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct MetadataWireLimits {
    /// Maximum number of input bytes accepted before parsing begins.
    max_json_bytes: usize,
}

impl MetadataWireLimits {
    /// Creates a JSON wire input limit.
    ///
    /// # Parameters
    ///
    /// * `max_json_bytes` - Largest accepted JSON input length in bytes.
    ///
    /// # Returns
    ///
    /// A limit that rejects inputs longer than `max_json_bytes`.
    #[inline(always)]
    pub const fn new(max_json_bytes: usize) -> Self {
        Self { max_json_bytes }
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
}

impl Default for MetadataWireLimits {
    /// Creates the standard one-mebibyte JSON input limit.
    #[inline(always)]
    fn default() -> Self {
        Self::new(DEFAULT_MAX_JSON_BYTES)
    }
}
