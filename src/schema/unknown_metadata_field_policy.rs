// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`UnknownMetadataFieldPolicy`] — handling for undeclared metadata keys.

use serde::{Deserialize, Serialize};

/// Policy for metadata fields that are not declared by a schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UnknownMetadataFieldPolicy {
    /// Reject metadata keys that are not declared in the schema.
    #[default]
    Reject,
    /// Allow metadata keys that are not declared in the schema.
    Allow,
}
