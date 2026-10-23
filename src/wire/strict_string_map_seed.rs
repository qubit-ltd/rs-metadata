// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seed decoder for bounded strict string-keyed maps.

use std::cell::RefCell;
use std::rc::Rc;

use serde::Deserialize;
use serde::de::DeserializeSeed;
use serde::de::Deserializer;

use super::internal::StrictStringMapValueVisitor;
use super::strict_string_map::StrictStringMap;
use crate::MetadataError;

/// Deserializes a strict string map with caller-provided resource bounds.
pub(crate) struct StrictStringMapSeed<'a, V> {
    /// Maximum number of entries accepted.
    max_entries: usize,
    /// Maximum UTF-8 key length accepted.
    max_key_bytes: usize,
    /// Optional slot retaining a structured wire-limit error.
    error_slot: Option<Rc<RefCell<Option<MetadataError>>>>,
    /// Lifetime marker for the deserializer borrow.
    lifetime: std::marker::PhantomData<&'a ()>,
    /// Marker for the seeded value type.
    marker: std::marker::PhantomData<fn() -> V>,
}

impl<V> StrictStringMapSeed<'static, V> {
    /// Creates a bounded strict-map seed.
    ///
    /// # Returns
    ///
    /// A seed configured with the maximum entry count and key length.
    #[must_use]
    #[inline]
    pub(crate) const fn new(max_entries: usize, max_key_bytes: usize) -> Self {
        Self {
            max_entries,
            max_key_bytes,
            error_slot: None,
            lifetime: std::marker::PhantomData,
            marker: std::marker::PhantomData,
        }
    }

    /// Attaches a request-local slot retaining value-free domain failures.
    ///
    /// # Returns
    ///
    /// The seed with the supplied error slot attached.
    #[cfg(all(feature = "json", feature = "schema"))]
    #[must_use]
    #[inline]
    pub(crate) fn with_error_slot(mut self, slot: Rc<RefCell<Option<MetadataError>>>) -> Self {
        self.error_slot = Some(slot);
        self
    }
}

impl<'de, 'a, V> DeserializeSeed<'de> for StrictStringMapSeed<'a, V>
where
    V: Deserialize<'de>,
{
    type Value = StrictStringMap<V>;

    /// Deserializes a map while enforcing caller-provided bounds.
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
            std::marker::PhantomData::<V>,
        ))
    }
}
