// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal schema deserialization helpers.

mod metadata_field_visitor;

pub(super) use metadata_field_visitor::deserialize_metadata_field;
