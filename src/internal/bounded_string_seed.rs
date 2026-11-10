// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Deserialization seed that validates strings before creating owned copies.

use std::fmt::Display;
use std::marker::PhantomData;

use serde::de;
use serde::de::DeserializeSeed;

use super::bounded_string_visitor::BoundedStringVisitor;

/// Validates a string before converting borrowed input to an owned string.
///
/// Borrowed and transient string events are copied only after validation
/// succeeds. An owned string is returned without another allocation after
/// validation. This does not change allocations made by an upstream parser or
/// by a deserializer that already supplies an owned string.
///
/// # Type Parameters
///
/// * `F` - One-shot callback that validates the string.
/// * `E` - Displayable validation error converted to the deserializer error.
pub(crate) struct BoundedStringSeed<F, E> {
    /// One-shot validator invoked for the next string event.
    validate: F,
    /// Carries the validator error type for deserialization error conversion.
    error: PhantomData<fn() -> E>,
}

impl<F, E> BoundedStringSeed<F, E> {
    /// Creates a seed with a validation callback.
    ///
    /// # Parameters
    ///
    /// * `validate` - Callback that accepts a string or returns a displayable
    ///   validation error.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Validator closure type.
    /// * `E` - Displayable validation error type, inferred from the callback.
    ///
    /// # Returns
    ///
    /// A seed that produces an owned `String` after validation.
    #[must_use]
    #[inline]
    pub(crate) fn new(validate: F) -> Self {
        Self {
            validate,
            error: PhantomData,
        }
    }
}

impl<'de, F, E> DeserializeSeed<'de> for BoundedStringSeed<F, E>
where
    F: FnOnce(&str) -> Result<(), E>,
    E: Display,
{
    /// The validated owned string.
    type Value = String;

    /// Validates and decodes a string using the configured callback.
    ///
    /// # Type Parameters
    ///
    /// * `D` - Deserializer supplying the string event.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Deserializer supplying a string event.
    ///
    /// # Errors
    ///
    /// Returns the callback's error if validation fails, or the deserializer
    /// error if the input is not a string.
    ///
    /// # Returns
    ///
    /// The accepted input as an owned string.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_string(BoundedStringVisitor {
            validate: self.validate,
            _error: PhantomData,
        })
    }
}
