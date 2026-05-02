/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`FilterMatchOptions`] — policies for filter evaluation.

use serde::{Deserialize, Serialize};

use crate::{MissingKeyPolicy, NumberComparisonPolicy};

/// Match policies used when evaluating a [`crate::MetadataFilter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FilterMatchOptions {
    /// Policy for missing keys in negative predicates.
    pub missing_key_policy: MissingKeyPolicy,
    /// Policy for mixed numeric comparisons.
    pub number_comparison_policy: NumberComparisonPolicy,
}
