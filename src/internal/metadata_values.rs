// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Redacted map view used by metadata diagnostics.

use std::collections::BTreeMap;

use qubit_redact::Redact;
use qubit_redact::RedactionWriter;
use qubit_redact::Sensitivity;
use qubit_value::Value;

/// Borrows metadata values and renders them through the redaction policy.
///
/// This view keeps the storage private while allowing [`crate::Metadata`] to
/// delegate its diagnostic map traversal to the shared redaction machinery.
pub(crate) struct MetadataValues<'a>(
    /// Values borrowed from the metadata object being rendered.
    pub(crate) &'a BTreeMap<String, Value>,
);

impl Redact for MetadataValues<'_> {
    /// Writes every admitted entry using low-sensitivity key classification.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        writer.map(|entries| {
            entries.for_each(self.0, |entries, (name, value)| {
                entries.sensitive_entry(Sensitivity::Low, name, || value);
            });
        });
    }
}
