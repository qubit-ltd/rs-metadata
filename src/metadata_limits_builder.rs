// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
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
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetadataLimitsBuilder {
    pub(crate) json_decode: JsonDecodeLimits,
    pub(crate) json_encode: JsonEncodeLimits,
    pub(crate) max_metadata_entries: usize,
    pub(crate) max_schema_fields: usize,
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

    /// Builds metadata limits by consuming this builder.
    #[inline]
    pub fn build(self) -> MetadataLimits {
        MetadataLimits::from_builder(self)
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
