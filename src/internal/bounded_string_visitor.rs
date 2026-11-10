// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Visitor that validates strings before creating owned copies.

use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

use serde::de;
use serde::de::Visitor;

/// Validates string events before making an owned copy.
///
/// Borrowed and transient strings are copied only after validation succeeds;
/// owned strings are returned after validation without another allocation.
///
/// # Type Parameters
///
/// * `F` - One-shot callback that validates the string.
/// * `E` - Displayable validation error converted to the deserializer error.
pub(super) struct BoundedStringVisitor<F, E> {
    /// One-shot validator invoked for the string event.
    pub(super) validate: F,
    /// Carries the validator error type for deserialization error conversion.
    pub(super) _error: PhantomData<fn() -> E>,
}

impl<'de, F, E> Visitor<'de> for BoundedStringVisitor<F, E>
where
    F: FnOnce(&str) -> Result<(), E>,
    E: Display,
{
    /// The validated string in owned form.
    type Value = String;

    /// Describes the only accepted input shape.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Formatter receiving the expected input description.
    ///
    /// # Returns
    ///
    /// A description indicating that a string is expected.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string")
    }

    /// Validates borrowed string data before allocating the owned result.
    ///
    /// # Type Parameters
    ///
    /// * `DE` - Deserializer error type used to report a validation failure.
    ///
    /// # Parameters
    ///
    /// * `value` - Borrowed decoded string supplied by the deserializer.
    ///
    /// # Errors
    ///
    /// Returns a deserializer error when validation rejects the string.
    ///
    /// # Returns
    ///
    /// Returns an owned copy after validation succeeds.
    fn visit_borrowed_str<DE>(self, value: &'de str) -> Result<Self::Value, DE>
    where
        DE: de::Error,
    {
        (self.validate)(value).map_err(DE::custom)?;
        Ok(value.to_owned())
    }

    /// Validates transient string data before allocating the owned result.
    ///
    /// # Type Parameters
    ///
    /// * `DE` - Deserializer error type used to report a validation failure.
    ///
    /// # Parameters
    ///
    /// * `value` - Transient decoded string supplied by the deserializer.
    ///
    /// # Errors
    ///
    /// Returns a deserializer error when validation rejects the string.
    ///
    /// # Returns
    ///
    /// Returns an owned copy after validation succeeds.
    fn visit_str<DE>(self, value: &str) -> Result<Self::Value, DE>
    where
        DE: de::Error,
    {
        (self.validate)(value).map_err(DE::custom)?;
        Ok(value.to_owned())
    }

    /// Validates an owned string and returns it without another allocation.
    ///
    /// # Type Parameters
    ///
    /// * `DE` - Deserializer error type used to report a validation failure.
    ///
    /// # Parameters
    ///
    /// * `value` - Owned string supplied by the deserializer.
    ///
    /// # Errors
    ///
    /// Returns a deserializer error when validation rejects the string.
    ///
    /// # Returns
    ///
    /// Returns the same owned string after validation succeeds.
    fn visit_string<DE>(self, value: String) -> Result<Self::Value, DE>
    where
        DE: de::Error,
    {
        (self.validate)(&value).map_err(DE::custom)?;
        Ok(value)
    }
}
