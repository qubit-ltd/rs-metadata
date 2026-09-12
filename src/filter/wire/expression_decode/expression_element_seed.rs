// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seed that checks one child collection item before reading its body.

use std::cell::RefCell;
use std::rc::Rc;

use qubit_budget::ResourceBudget;
use serde::de::DeserializeSeed;
use serde::de::Deserializer;

use super::super::filter_expression_wire_v1::FilterExpressionWireV1;
use super::super::filter_expression_wire_v1_seed::FilterExpressionWireV1Seed;
use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::MetadataError;

/// Seed that checks one child collection item before reading its body.
pub(in crate::filter::wire) struct ExpressionElementSeed<'a> {
    /// Receiver-side AST limits.
    pub(in crate::filter::wire) receiver_limits: FilterLimits,
    /// Remaining node budget.
    pub(in crate::filter::wire) node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
    /// Child expression depth.
    pub(in crate::filter::wire) depth: usize,
    /// First structured limit error.
    pub(in crate::filter::wire) error_slot: Rc<RefCell<Option<MetadataError>>>,
}

impl<'de, 'a> DeserializeSeed<'de> for ExpressionElementSeed<'a> {
    type Value = FilterExpressionWireV1;

    /// Charges one child item before delegating to the recursive expression
    /// seed.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        FilterExpressionWireV1Seed::new(self.receiver_limits, self.node_budget, self.depth, self.error_slot)
            .deserialize(deserializer)
    }
}
