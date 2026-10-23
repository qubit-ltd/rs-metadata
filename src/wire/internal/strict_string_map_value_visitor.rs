// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Visitor that decodes bounded strict string maps using a caller-provided
//! seed.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt;
use std::rc::Rc;

use serde::de;
use serde::de::DeserializeSeed;
use serde::de::MapAccess;
use serde::de::Visitor;

use super::super::strict_string_map::StrictStringMap;
use crate::MetadataError;
use crate::MetadataWireLimitKind;

/// Builds a strict map while enforcing key, count, and value decoding limits.
///
/// # Type Parameters
///
/// `S` is the deserialization seed cloned for each map value.
pub(in crate::wire) struct StrictStringMapValueVisitor<S> {
    /// Maximum number of entries accepted.
    max_entries: usize,
    /// Maximum UTF-8 key length accepted.
    max_key_bytes: usize,
    /// Optional structured error slot.
    error_slot: Option<Rc<RefCell<Option<MetadataError>>>>,
    /// Seed used to decode each value.
    value_seed: S,
}

impl<S> StrictStringMapValueVisitor<S> {
    /// Creates a visitor with the bounds and value seed used for decoding.
    ///
    /// # Parameters
    ///
    /// * `max_entries` - Maximum number of map entries accepted.
    /// * `max_key_bytes` - Maximum UTF-8 byte length accepted for each key.
    /// * `error_slot` - Optional slot for retaining structured wire-limit
    ///   errors.
    /// * `value_seed` - Seed cloned to decode each map value.
    ///
    /// # Returns
    ///
    /// A visitor configured with the supplied bounds and value seed.
    #[must_use]
    #[inline]
    pub(in crate::wire) fn new(
        max_entries: usize,
        max_key_bytes: usize,
        error_slot: Option<Rc<RefCell<Option<MetadataError>>>>,
        value_seed: S,
    ) -> Self {
        Self {
            max_entries,
            max_key_bytes,
            error_slot,
            value_seed,
        }
    }
}

impl<'de, S> Visitor<'de> for StrictStringMapValueVisitor<S>
where
    S: DeserializeSeed<'de> + Clone,
{
    type Value = StrictStringMap<S::Value>;

    /// Describes the expected input shape.
    ///
    /// # Returns
    ///
    /// A description of a map with unique string keys.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a map with unique string keys")
    }

    /// Decodes entries while enforcing key, count, and value bounds.
    ///
    /// # Errors
    ///
    /// Returns the map access error if a key or value cannot be decoded, a key
    /// is repeated, or either configured map limit is exceeded.
    ///
    /// # Returns
    ///
    /// The decoded strict string map when every key and value satisfies the
    /// configured limits and all keys are unique.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = BTreeMap::new();
        while let Some(key) = map.next_key::<String>()? {
            if key.len() > self.max_key_bytes {
                if let Some(slot) = &self.error_slot {
                    *slot.borrow_mut() = Some(MetadataError::WireLimitExceeded {
                        kind: MetadataWireLimitKind::KeyBytes,
                        value: key.len(),
                        maximum: self.max_key_bytes,
                    });
                }
                return Err(de::Error::custom(format!(
                    "map key has {} bytes, exceeding the limit of {} bytes",
                    key.len(),
                    self.max_key_bytes,
                )));
            }
            if values.len() >= self.max_entries {
                if let Some(slot) = &self.error_slot {
                    *slot.borrow_mut() = Some(MetadataError::WireLimitExceeded {
                        kind: MetadataWireLimitKind::Entries,
                        value: values.len() + 1,
                        maximum: self.max_entries,
                    });
                }
                return Err(de::Error::custom(format!(
                    "map has more than the limit of {} entries",
                    self.max_entries,
                )));
            }
            match values.entry(key) {
                Entry::Occupied(entry) => {
                    return Err(de::Error::custom(format!("duplicate map key '{}'", entry.key())));
                }
                Entry::Vacant(entry) => {
                    entry.insert(map.next_value_seed(self.value_seed.clone())?);
                }
            }
        }
        Ok(StrictStringMap(values))
    }
}
