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

/// A string-keyed map that rejects duplicate keys while deserializing.
pub(crate) struct StrictStringMap<V>(pub(crate) BTreeMap<String, V>);

impl<V> StrictStringMap<V> {
    /// Returns the decoded entries.
    ///
    /// # Returns
    ///
    /// The complete map after duplicate-key validation.
    pub(crate) fn into_inner(self) -> BTreeMap<String, V> {
        self.0
    }
}

impl<'de, V> Deserialize<'de> for StrictStringMap<V>
where
    V: Deserialize<'de>,
{
    /// Deserializes a map while rejecting a repeated key.
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
