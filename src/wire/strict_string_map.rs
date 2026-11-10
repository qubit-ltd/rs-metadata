// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A strict string-keyed map for wire-format decoding.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Deserializer;
use serde::de::DeserializeSeed;

/// A string-keyed map that rejects duplicate keys and enforces wire-format
/// entry-count and key-length limits while deserializing.
///
/// # Type Parameters
///
/// * `V` - Decoded value stored for each string key.
pub(crate) struct StrictStringMap<V>(
    /// Entries decoded after duplicate-key validation.
    pub(crate) BTreeMap<String, V>,
);

impl<V> StrictStringMap<V> {
    /// Consumes this wrapper and returns its decoded entries without further
    /// validation.
    ///
    /// # Returns
    ///
    /// The map whose keys were checked for duplicates during deserialization.
    #[must_use]
    #[inline]
    pub(crate) fn into_inner(self) -> BTreeMap<String, V> {
        self.0
    }
}

impl<'de, V> Deserialize<'de> for StrictStringMap<V>
where
    V: Deserialize<'de>,
{
    /// Deserializes a strict string-keyed map using the configured wire limits.
    ///
    /// # Type Parameters
    ///
    /// * `D` - Deserializer providing the map.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Source of the serialized map.
    ///
    /// # Returns
    ///
    /// The decoded map after duplicate-key and resource-limit validation.
    ///
    /// # Errors
    ///
    /// Returns the deserializer error if an entry cannot be decoded, a key is
    /// repeated, or the map exceeds the configured entry-count or key-length
    /// limit.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::strict_string_map_seed::StrictStringMapSeed::new(
            crate::constants::STRICT_STRING_MAP_MAX_ENTRIES,
            crate::constants::STRICT_STRING_MAP_MAX_KEY_BYTES,
        )
        .deserialize(deserializer)
    }
}
