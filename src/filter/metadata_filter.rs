// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataFilter`].

#[cfg(feature = "json")]
use std::cell::RefCell;
#[cfg(feature = "json")]
use std::io::Write;
#[cfg(feature = "json")]
use std::rc::Rc;

#[cfg(feature = "json")]
use qubit_budget::MeasuredBudgetError;
#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeSession;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeSession;
#[cfg(feature = "json")]
use qubit_budget::json::JsonResource;
#[cfg(feature = "json")]
use qubit_json::text::JsonDecodeError;
#[cfg(feature = "json")]
use qubit_json::text::decode_admitted_slice_seed;
#[cfg(feature = "json")]
use qubit_json::text::encode_to_vec;
#[cfg(feature = "json")]
use qubit_json::text::encode_to_writer;
use qubit_utils::Transient;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de;
#[cfg(feature = "json")]
use serde::de::Error as DeError;
use serde::ser::Error as SerError;

use super::metadata_filter_builder::MetadataFilterBuilder;
use super::wire::MetadataFilterWireV1;
use super::wire::MetadataFilterWireV1Ref;
#[cfg(feature = "json")]
use super::wire::MetadataFilterWireV1Seed;
#[cfg(feature = "schema")]
use crate::Condition;
use crate::FilterExpression;
use crate::FilterLimits;
use crate::FilterMatchOptions;
use crate::Metadata;
#[cfg(feature = "schema")]
use crate::MetadataResult;
#[cfg(feature = "json")]
use crate::metadata_limits::MetadataLimits;

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
    limits: Transient<FilterLimits>,
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

    /// Deserializes a filter and validates a receiver-controlled AST bound.
    ///
    /// The AST is materialized by the underlying deserializer before the
    /// receiver bound is checked. Callers must separately bound raw input
    /// bytes and any deserializer-specific resources before accepting
    /// untrusted data.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Source of the versioned filter representation.
    /// * `receiver_limits` - Local upper bounds applied to the decoded AST.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error for malformed V1 data or an unsupported
    /// version. The underlying deserializer's error type receives the
    /// structured filter-limit or policy failure through `D::Error::custom`;
    /// this generic API cannot return [`crate::MetadataWireDecodeError`]
    /// directly.
    pub fn deserialize_with_filter_limits<'de, D>(
        deserializer: D,
        receiver_limits: FilterLimits,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        MetadataFilterWireV1::deserialize(deserializer)?
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
    /// Returns an input-size error before parsing, a nested-value limit error,
    /// a structured filter-contract error, or `InvalidJson` for malformed
    /// strict filter input and receiver-limit failures found during
    /// incremental decoding.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice(
        input: &[u8],
    ) -> Result<Self, crate::MetadataWireDecodeError> {
        Self::decode_json_slice_with_limits(
            input,
            MetadataLimits::default(),
            FilterLimits::MAX,
        )
    }

    /// Decodes a strict metadata-filter JSON envelope after validating both
    /// wire-byte and receiver-controlled AST limits.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete untrusted JSON input.
    /// * `limits` - Shared JSON limits for this decoding session.
    /// * `receiver_filter_limits` - Local AST limits validated after decoding.
    ///
    /// # Returns
    ///
    /// The decoded filter constrained by receiver-controlled limits.
    ///
    /// # Errors
    ///
    /// Returns an input-size error before parsing, a structured filter-contract
    /// error in `Filter`, or `InvalidJson` for syntax, strict-envelope, nested
    /// value, and receiver-limit failures. Receiver AST limits are charged
    /// while the expression tree is read; generic JSON traversal is handled by
    /// the shared budget adapter. Individual
    /// JSON strings and embedded value payloads remain bounded by the outer
    /// input-byte limit.
    #[cfg(feature = "json")]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        limits: MetadataLimits,
        receiver_filter_limits: FilterLimits,
    ) -> Result<Self, crate::MetadataWireDecodeError> {
        limits
            .validate()
            .map_err(crate::MetadataWireDecodeError::InvalidJson)?;
        let mut session = JsonDecodeSession::owned(limits.json_decode());
        let error_slot = Rc::new(RefCell::new(None));
        let wire = decode_admitted_slice_seed(
            MetadataFilterWireV1Seed::new(
                receiver_filter_limits,
                Rc::clone(&error_slot),
            ),
            input,
            &mut session,
        )
        .map_err(|error| {
            error_slot.borrow_mut().take().map_or_else(
                || filter_json_error(error),
                crate::MetadataWireDecodeError::Filter,
            )
        })?;
        wire.into_filter(receiver_filter_limits)
            .map_err(crate::MetadataWireDecodeError::Filter)
    }

    /// Encodes this filter with the default JSON budget profile.
    #[cfg(feature = "json")]
    pub fn to_json_vec(
        &self,
    ) -> Result<Vec<u8>, crate::MetadataWireEncodeError> {
        self.to_json_vec_with_limits(
            crate::metadata_limits::default_json_encode_limits(),
        )
    }

    /// Encodes this filter with caller-provided JSON budgets.
    ///
    /// # Parameters
    ///
    /// * `limits` - Output and JSON-value budgets for this operation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::MetadataWireEncodeError`] when encoding exceeds a
    /// budget or the filter cannot be represented by the V1 wire format.
    #[cfg(feature = "json")]
    pub fn to_json_vec_with_limits(
        &self,
        limits: JsonEncodeLimits,
    ) -> Result<Vec<u8>, crate::MetadataWireEncodeError> {
        let mut session = JsonEncodeSession::owned(limits);
        encode_to_vec(self, &mut session).map_err(Into::into)
    }

    /// Encodes this filter to a writer with the default JSON budget profile.
    #[cfg(feature = "json")]
    pub fn to_json_writer<W>(
        &self,
        writer: W,
    ) -> Result<(), crate::MetadataWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(
            writer,
            crate::metadata_limits::default_json_encode_limits(),
        )
    }

    /// Encodes this filter to a writer with caller-provided JSON budgets.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the compact JSON document.
    /// * `limits` - Output and JSON-value budgets for this operation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::MetadataWireEncodeError`] when encoding exceeds a
    /// budget, the filter cannot be represented by the V1 wire format, or the
    /// writer rejects the output.
    #[cfg(feature = "json")]
    pub fn to_json_writer_with_limits<W>(
        &self,
        writer: W,
        limits: JsonEncodeLimits,
    ) -> Result<(), crate::MetadataWireEncodeError>
    where
        W: Write,
    {
        let mut session = JsonEncodeSession::owned(limits);
        encode_to_writer(writer, self, &mut session).map_err(Into::into)
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
            limits: Transient::new(limits),
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
        *self.limits.get()
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
        MetadataFilterWireV1Ref::try_from(self)
            .map_err(<S::Error as SerError>::custom)?
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

/// Converts a shared JSON adapter error into the filter decoding error.
#[cfg(feature = "json")]
fn filter_json_error(
    error: JsonDecodeError<JsonResource>,
) -> crate::MetadataWireDecodeError {
    match error {
        JsonDecodeError::Budget(error) => match error {
            MeasuredBudgetError::Budget(error) => {
                crate::MetadataWireDecodeError::Budget(error)
            }
            MeasuredBudgetError::Quantity { resource, source } => {
                crate::MetadataWireDecodeError::Quantity { resource, source }
            }
        },
        JsonDecodeError::Syntax(error) => {
            crate::MetadataWireDecodeError::Syntax(error)
        }
        JsonDecodeError::Deserialize(error) => {
            crate::MetadataWireDecodeError::InvalidJson(
                <serde_json::Error as DeError>::custom(error),
            )
        }
    }
}
