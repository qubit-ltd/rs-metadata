// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed schema-field diagnostic adapter.

use qubit_redact::Redact;
use qubit_redact::RedactionWriter;

use crate::schema::MetadataField;

/// A borrowed field definition formatted through the active redaction writer.
pub(crate) struct SchemaFieldDiagnostic<'a>(
    /// Borrowed field definition exposed through redacted diagnostics.
    pub(crate) &'a MetadataField,
);

impl Redact for SchemaFieldDiagnostic<'_> {
    /// Writes the field's public type and required status.
    ///
    /// # Parameters
    ///
    /// * `writer` - Redaction writer receiving the field representation.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        writer.record("MetadataField", |fields| {
            fields.unmarked("data_type", || self.0.data_type());
            fields.unmarked("required", || self.0.is_required());
        });
    }
}
