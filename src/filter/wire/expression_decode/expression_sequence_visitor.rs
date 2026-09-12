// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Visitor for a bounded child-expression sequence.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use qubit_budget::ResourceBudget;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::super::filter_expression_wire_v1::FilterExpressionWireV1;
use super::expression_element_seed::ExpressionElementSeed;
use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::MetadataError;

/// Visitor for a bounded child-expression sequence.
pub(in crate::filter::wire) struct ExpressionSequenceVisitor<'a> {
    /// Receiver-side AST limits.
    pub(in crate::filter::wire) receiver_limits: FilterLimits,
    /// Remaining node budget.
    pub(in crate::filter::wire) node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
    /// Child expression depth.
    pub(in crate::filter::wire) depth: usize,
    /// First structured limit error.
    pub(in crate::filter::wire) error_slot: Rc<RefCell<Option<MetadataError>>>,
}

impl<'de, 'a> Visitor<'de> for ExpressionSequenceVisitor<'a> {
    type Value = Vec<FilterExpressionWireV1>;

    /// Describes a sequence of filter expression nodes.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a sequence of filter expressions")
    }

    /// Decodes children one by one and charges each collection item first.
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let maximum = self.receiver_limits.max_nodes();
        let capacity = sequence.size_hint().unwrap_or(0).min(maximum);
        let mut children = Vec::with_capacity(capacity);
        while let Some(child) = sequence.next_element_seed(ExpressionElementSeed {
            receiver_limits: self.receiver_limits,
            node_budget: &mut *self.node_budget,
            depth: self.depth,
            error_slot: Rc::clone(&self.error_slot),
        })? {
            children.push(child);
        }
        Ok(children)
    }
}
