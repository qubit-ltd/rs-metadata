// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`UnknownFilterFieldPolicy`] — handling for undeclared filter keys.

use serde::{Deserialize, Serialize};

/// Policy for filter fields that are not declared by a schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UnknownFilterFieldPolicy {
    /// Reject filter keys that are not declared in the schema.
    #[default]
    Reject,
    /// Accept undeclared filter keys without type or operator validation.
    AllowUnchecked,
}
