// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seed decoder for bounded strict string-keyed maps with caller-provided
//! values.

use std::cell::RefCell;
use std::rc::Rc;

use serde::de::DeserializeSeed;
use serde::de::Deserializer;

use super::internal::StrictStringMapValueVisitor;
use super::strict_string_map::StrictStringMap;
use crate::MetadataError;

/// Deserializes a strict string map with a caller-provided value seed.
///
/// # Type Parameters
///
/// `S` is the seed used to deserialize each map value.
pub(crate) struct StrictStringMapValueSeed<S> {
    /// Maximum number of entries accepted.
    max_entries: usize,
    /// Maximum UTF-8 key length accepted.
    max_key_bytes: usize,
    /// Optional slot retaining a structured wire-limit error.
    error_slot: Option<Rc<RefCell<Option<MetadataError>>>>,
    /// Seed used to decode each map value.
    value_seed: S,
}

impl<S> StrictStringMapValueSeed<S> {
    /// Creates a bounded strict-map seed with explicit value construction.
    ///
    /// # Parameters
    ///
    /// * `max_entries` - Maximum number of map entries accepted.
    /// * `max_key_bytes` - Maximum UTF-8 byte length accepted for each key.
    /// * `value_seed` - Seed used to deserialize each map value.
    ///
    /// # Returns
    ///
    /// A seed configured with the entry limit, key-length limit, and value
    /// seed.
    #[must_use]
    #[inline]
    pub(crate) const fn new(max_entries: usize, max_key_bytes: usize, value_seed: S) -> Self {
        Self {
            max_entries,
            max_key_bytes,
            error_slot: None,
            value_seed,
        }
    }

    /// Attaches a request-local slot retaining value-free domain failures.
    ///
    /// # Parameters
    ///
    /// * `slot` - Shared slot that receives a structured wire-limit error.
    ///
    /// # Returns
    ///
    /// The seed with the supplied error slot attached.
    #[must_use]
    #[cfg(feature = "json")]
    #[inline]
    pub(crate) fn with_error_slot(mut self, slot: Rc<RefCell<Option<MetadataError>>>) -> Self {
        self.error_slot = Some(slot);
        self
    }
}

impl<'de, S> DeserializeSeed<'de> for StrictStringMapValueSeed<S>
where
    S: DeserializeSeed<'de> + Clone,
{
    type Value = StrictStringMap<S::Value>;

    /// Deserializes a map while applying the supplied seed to every value.
    ///
    /// # Errors
    ///
    /// Returns the deserializer error if a key or value cannot be decoded, a
    /// key is repeated, or either configured map limit is exceeded.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(StrictStringMapValueVisitor::new(
            self.max_entries,
            self.max_key_bytes,
            self.error_slot,
            self.value_seed,
        ))
    }
}
