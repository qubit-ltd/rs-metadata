/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`UnknownFieldPolicy`] — handling for schema-unknown metadata and filter keys.

use serde::{Deserialize, Serialize};

/// Policy for fields that appear in metadata or filters but are not declared by
/// a schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UnknownFieldPolicy {
    /// Reject fields that are not declared in the schema.
    #[default]
    Reject,
    /// Allow fields that are not declared in the schema.
    Allow,
}
