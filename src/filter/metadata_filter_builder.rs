// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Builder for [`crate::MetadataFilter`].

use crate::{
    FilterExpression,
    FilterLimits,
    FilterMatchOptions,
    MetadataError,
    MetadataFilter,
    MetadataResult,
};
#[cfg(feature = "schema")]
use crate::{
    MetadataSchema,
    MetadataValidationError,
    MetadataValidationResult,
};

/// Builder for the non-logical configuration of a [`MetadataFilter`].
///
/// The expression is mandatory. Repeated property calls are intentional and
/// use last-write-wins semantics.
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataFilterBuilder {
    /// Root expression supplied by the caller.
    expression: Option<FilterExpression>,
    /// Match options for the built filter.
    options: FilterMatchOptions,
    /// Resource limits stored with the built filter.
    limits: FilterLimits,
}

impl MetadataFilterBuilder {
    /// Creates a builder with default options and library hard limits.
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self {
            expression: None,
            options: FilterMatchOptions::builder().build(),
            limits: FilterLimits::MAX,
        }
    }

    /// Sets the required root expression.
    #[inline(always)]
    #[must_use]
    pub fn expression(mut self, expression: FilterExpression) -> Self {
        self.expression = Some(expression);
        self
    }

    /// Sets the match options.
    #[inline(always)]
    #[must_use]
    pub fn options(mut self, options: FilterMatchOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets the resource limits.
    #[inline(always)]
    #[must_use]
    pub fn limits(mut self, limits: FilterLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Builds a filter after validating its expression against its limits.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::MissingFilterExpression`] if no expression was
    /// supplied, or [`MetadataError::FilterLimitExceeded`] if it exceeds the
    /// configured limits.
    #[inline]
    pub fn build(self) -> MetadataResult<MetadataFilter> {
        let expression = self
            .expression
            .ok_or(MetadataError::MissingFilterExpression)?;
        expression.validate_limits(self.limits)?;
        Ok(MetadataFilter::new(expression, self.options, self.limits))
    }

    /// Builds and validates a filter against `schema`.
    ///
    /// # Errors
    ///
    /// Returns construction errors or aggregate schema-validation errors.
    #[cfg(feature = "schema")]
    #[inline]
    pub fn build_checked(
        self,
        schema: &MetadataSchema,
    ) -> MetadataValidationResult<MetadataFilter> {
        let filter =
            self.build().map_err(MetadataValidationError::from_issue)?;
        schema.validate_filter(&filter)?;
        Ok(filter)
    }
}

impl Default for MetadataFilterBuilder {
    /// Returns a builder with default options and library hard limits.
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
