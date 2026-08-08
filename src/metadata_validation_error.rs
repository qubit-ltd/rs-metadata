// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataValidationError`] — aggregate schema validation failures.

use std::fmt;

use crate::metadata_error::MetadataError;

/// Aggregate error returned by schema-level validation APIs.
///
/// Unlike single-entry metadata accessors, schema validation can discover
/// multiple independent issues in one pass. This type preserves all collected
/// [`MetadataError`] values so callers can report or fix them together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataValidationError {
    /// Collected validation issues.
    issues: Vec<MetadataError>,
}

impl MetadataValidationError {
    /// Creates an aggregate validation error from one issue.
    ///
    /// # Parameters
    ///
    /// * `issue` - Validation issue to store.
    ///
    /// # Returns
    ///
    /// An aggregate error containing exactly one issue.
    #[inline]
    #[must_use]
    pub fn from_issue(issue: MetadataError) -> Self {
        Self {
            issues: vec![issue],
        }
    }

    /// Creates an aggregate validation error from a non-empty issue list.
    ///
    /// Returns `None` when `issues` is empty because an empty validation error
    /// has no actionable meaning.
    ///
    /// # Parameters
    ///
    /// * `issues` - Validation issues to aggregate.
    ///
    /// # Returns
    ///
    /// `Some` aggregate error for a non-empty list; otherwise, `None`.
    #[inline]
    pub fn from_issues(issues: Vec<MetadataError>) -> Option<Self> {
        (!issues.is_empty()).then_some(Self { issues })
    }

    /// Returns the collected validation issues.
    ///
    /// # Returns
    ///
    /// The issues in discovery order.
    #[inline(always)]
    #[must_use]
    pub fn issues(&self) -> &[MetadataError] {
        &self.issues
    }

    /// Returns the number of collected validation issues.
    ///
    /// # Returns
    ///
    /// The issue count.
    #[inline(always)]
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.issues.len()
    }

    /// Converts this aggregate error into its collected issues.
    ///
    /// # Returns
    ///
    /// The owned issues in discovery order.
    #[inline(always)]
    #[must_use]
    pub fn into_issues(self) -> Vec<MetadataError> {
        self.issues
    }
}

impl fmt::Display for MetadataValidationError {
    /// Formats the validation summary and every collected issue.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} metadata validation issue(s)",
            self.issues.len()
        )?;
        for (index, issue) in self.issues.iter().enumerate() {
            write!(formatter, "; {}: {issue}", index + 1)?;
        }
        Ok(())
    }
}

impl std::error::Error for MetadataValidationError {}
