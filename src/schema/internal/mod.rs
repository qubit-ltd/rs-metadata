// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal schema deserialization helpers.

mod metadata_field_visitor;
mod schema_field_diagnostic;
mod schema_fields;

pub(super) use metadata_field_visitor::deserialize_metadata_field;
pub(crate) use schema_field_diagnostic::SchemaFieldDiagnostic;
pub(crate) use schema_fields::SchemaFields;
