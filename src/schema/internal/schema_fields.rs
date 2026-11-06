// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed, admission-controlled schema fields map adapter.

use std::collections::BTreeMap;

use qubit_redact::Redact;
use qubit_redact::RedactionWriter;

use crate::schema::MetadataField;
use crate::schema::internal::SchemaFieldDiagnostic;

/// A borrowed schema map whose entries share the active budget.
pub(crate) struct SchemaFields<'a>(
    /// Borrowed field map rendered under the active collection budget.
    pub(crate) &'a BTreeMap<String, MetadataField>,
);

impl Redact for SchemaFields<'_> {
    /// Writes only entries admitted by the current collection budget.
    ///
    /// # Parameters
    ///
    /// * `writer` - Redaction writer receiving the map representation.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        writer.map(|entries| {
            entries.for_each(self.0, |entries, (key, value)| {
                entries.nested_entry(key, &SchemaFieldDiagnostic(value));
            });
        });
    }
}
