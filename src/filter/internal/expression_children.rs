// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed sequence adapter for filter-expression diagnostics.

use qubit_redact::Redact;
use qubit_redact::RedactionWriter;

use crate::FilterExpression;

/// A borrowed set of expression children written under one redaction session.
/// Borrows the expression children written in one redaction session.
pub(crate) struct ExpressionChildren<'a>(
    /// Expression children written in one redaction session.
    pub(crate) &'a [FilterExpression],
);

impl Redact for ExpressionChildren<'_> {
    /// Writes admitted children without cloning them or resetting the budget.
    /// # Parameters
    ///
    /// * `writer` - Destination that receives the encoded or redacted output.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        writer.sequence(|items| {
            items.for_each(self.0, |items, child| {
                items.nested_item(child);
            });
        });
    }
}
