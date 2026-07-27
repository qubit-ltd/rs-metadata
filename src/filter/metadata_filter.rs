// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataFilter`].

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de,
};

use super::metadata_filter_builder::MetadataFilterBuilder;
use super::wire::MetadataFilterWire;
#[cfg(feature = "schema")]
use crate::{
    Condition,
    MetadataResult,
};
use crate::{
    FilterExpression,
    FilterLimits,
    FilterMatchOptions,
    Metadata,
};

/// An expression, its matching policy, and its resource limits.
///
/// Boolean composition belongs to [`FilterExpression`]. This type only binds
/// an already-built expression to the options and limits used to evaluate it.
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataFilter {
    /// Root Boolean expression.
    expression: FilterExpression,
    /// Evaluation options.
    options: FilterMatchOptions,
    /// Bounds that the expression satisfies.
    limits: FilterLimits,
}

impl MetadataFilter {
    /// Creates a filter that matches every metadata object.
    ///
    /// # Returns
    ///
    /// A constant-true filter using default match options and library hard
    /// limits.
    #[inline]
    #[must_use = "the constructed all-matching filter should be used"]
    pub fn all() -> Self {
        Self::new(
            FilterExpression::match_all(),
            FilterMatchOptions::default(),
            FilterLimits::MAX,
        )
    }

    /// Creates a filter that matches no metadata object.
    ///
    /// # Returns
    ///
    /// A constant-false filter using default match options and library hard
    /// limits.
    #[inline]
    #[must_use = "the constructed no-match filter should be used"]
    pub fn none() -> Self {
        Self::new(
            FilterExpression::match_none(),
            FilterMatchOptions::default(),
            FilterLimits::MAX,
        )
    }

    /// Creates a builder for a metadata filter.
    #[inline(always)]
    #[must_use]
    pub const fn builder() -> MetadataFilterBuilder {
        MetadataFilterBuilder::new()
    }

    /// Deserializes a filter while enforcing a receiver-controlled AST bound.
    ///
    /// This method limits only the decoded AST. Callers must separately bound
    /// the raw input byte length before deserializing untrusted data.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Source of the versioned filter representation.
    /// * `receiver_limits` - Local upper bounds applied to the decoded AST.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error for malformed v4 data, an unsupported
    /// version, or an expression exceeding either the sender-declared limits
    /// or `receiver_limits`.
    pub fn deserialize_with_filter_limits<'de, D>(
        deserializer: D,
        receiver_limits: FilterLimits,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        MetadataFilterWire::deserialize(deserializer)?
            .into_filter(receiver_limits)
            .map_err(de::Error::custom)
    }

    /// Decodes a strict metadata-filter JSON envelope using the default wire
    /// byte and receiver AST limits.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete untrusted JSON input.
    ///
    /// # Returns
    ///
    /// The decoded filter.
    ///
    /// # Errors
    ///
    /// Returns an input-size error before parsing or an invalid-JSON error for
    /// malformed strict filter input or AST limits rejected after decoding.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice(
        input: &[u8],
    ) -> Result<Self, crate::MetadataWireDecodeError> {
        Self::decode_json_slice_with_limits(
            input,
            crate::MetadataWireLimits::default(),
            FilterLimits::MAX,
        )
    }

    /// Decodes a strict metadata-filter JSON envelope after enforcing both
    /// wire-byte and receiver-controlled AST limits.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete untrusted JSON input.
    /// * `wire_limits` - Byte limit checked before JSON parsing.
    /// * `receiver_filter_limits` - Local AST limits applied after decoding.
    ///
    /// # Returns
    ///
    /// The decoded filter constrained by the sender and receiver limits.
    ///
    /// # Errors
    ///
    /// Returns an input-size error before parsing or an invalid-JSON error for
    /// syntax, strict-envelope, sender-limit, or receiver-limit failures.
    #[cfg(feature = "json")]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        wire_limits: crate::MetadataWireLimits,
        receiver_filter_limits: FilterLimits,
    ) -> Result<Self, crate::MetadataWireDecodeError> {
        wire_limits.check_json_bytes(input.len())?;
        let mut deserializer = serde_json::Deserializer::from_slice(input);
        let filter = Self::deserialize_with_filter_limits(
            &mut deserializer,
            receiver_filter_limits,
        )
        .map_err(crate::MetadataWireDecodeError::InvalidJson)?;
        deserializer
            .end()
            .map_err(crate::MetadataWireDecodeError::InvalidJson)?;
        Ok(filter)
    }

    /// Creates a filter from already validated parts.
    #[inline(always)]
    pub(crate) const fn new(
        expression: FilterExpression,
        options: FilterMatchOptions,
        limits: FilterLimits,
    ) -> Self {
        Self {
            expression,
            options,
            limits,
        }
    }

    /// Returns the root expression.
    #[inline(always)]
    #[must_use = "the filter expression should be inspected"]
    pub const fn expression(&self) -> &FilterExpression {
        &self.expression
    }

    /// Returns the evaluation options.
    #[inline(always)]
    #[must_use]
    pub const fn options(&self) -> FilterMatchOptions {
        self.options
    }

    /// Returns the resource limits.
    #[inline(always)]
    #[must_use = "the filter limits should be inspected"]
    pub const fn limits(&self) -> FilterLimits {
        self.limits
    }

    /// Returns whether `metadata` satisfies this filter.
    #[inline(always)]
    #[must_use]
    pub fn matches(&self, metadata: &Metadata) -> bool {
        self.expression.evaluate(metadata, self.options).is_match()
    }

    /// Visits every leaf condition in the expression.
    ///
    /// # Errors
    ///
    /// Returns the first error produced by `visitor`.
    #[cfg(feature = "schema")]
    #[inline(always)]
    pub(crate) fn visit_conditions<F>(
        &self,
        mut visitor: F,
    ) -> MetadataResult<()>
    where
        F: FnMut(&Condition) -> MetadataResult<()>,
    {
        self.expression.visit_conditions(&mut visitor)
    }
}

impl Serialize for MetadataFilter {
    /// Serializes this filter through its versioned wire representation.
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        MetadataFilterWire::try_from(self)
            .map_err(<S::Error as serde::ser::Error>::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MetadataFilter {
    /// Deserializes a filter using library hard limits as receiver limits.
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::deserialize_with_filter_limits(deserializer, FilterLimits::MAX)
    }
}
