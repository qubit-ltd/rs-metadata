// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seed that bounds a sequence of child expressions.

use std::cell::RefCell;
use std::rc::Rc;

use qubit_budget::ResourceBudget;
use serde::de::DeserializeSeed;
use serde::de::Deserializer;

use super::super::filter_expression_wire_v1::FilterExpressionWireV1;
use super::expression_sequence_visitor::ExpressionSequenceVisitor;
use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::MetadataError;

/// Seed that bounds a sequence of child expressions.
pub(in crate::filter::wire) struct ExpressionSequenceSeed<'a> {
    /// Receiver-side AST limits.
    pub(in crate::filter::wire) receiver_limits: FilterLimits,
    /// Remaining node budget.
    pub(in crate::filter::wire) node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
    /// Parent expression depth.
    pub(in crate::filter::wire) depth: usize,
    /// First structured limit error.
    pub(in crate::filter::wire) error_slot: Rc<RefCell<Option<MetadataError>>>,
}

impl<'de, 'a> DeserializeSeed<'de> for ExpressionSequenceSeed<'a> {
    type Value = Vec<FilterExpressionWireV1>;

    /// Decodes child expressions with bounded growth.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExpressionSequenceVisitor {
            receiver_limits: self.receiver_limits,
            node_budget: self.node_budget,
            depth: self.depth,
            error_slot: self.error_slot,
        })
    }
}
