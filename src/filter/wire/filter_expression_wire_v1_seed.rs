// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Incremental V1 filter-expression decoding with receiver limits.

use std::cell::RefCell;
use std::rc::Rc;

use qubit_budget::ResourceBudget;
use serde::de;
use serde::de::DeserializeSeed;
use serde::de::Deserializer;

use super::FilterExpressionWireV1;
use super::expression_decode::ExpressionVisitor;
use super::expression_decode::capture_filter_error;
use super::expression_decode::filter_limit_error;
use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::MetadataError;

/// Seed that decodes one expression while charging filter-domain budgets.
pub(crate) struct FilterExpressionWireV1Seed<'a> {
    /// Receiver-side AST limits.
    receiver_limits: FilterLimits,
    /// Remaining node budget for this decode operation.
    node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
    /// Current expression depth.
    depth: usize,
    /// Slot retaining the first structured limit error.
    error_slot: Rc<RefCell<Option<MetadataError>>>,
}

impl<'a> FilterExpressionWireV1Seed<'a> {
    /// Creates a seed for one expression at `depth`.
    pub(crate) const fn new(
        receiver_limits: FilterLimits,
        node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
        depth: usize,
        error_slot: Rc<RefCell<Option<MetadataError>>>,
    ) -> Self {
        Self {
            receiver_limits,
            node_budget,
            depth,
            error_slot,
        }
    }

    /// Charges one expression node before reading its map body.
    fn enter_node<E>(&mut self) -> Result<(), E>
    where
        E: de::Error,
    {
        if self.depth > self.receiver_limits.max_depth() {
            let error = MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::Depth,
                value: self.depth,
                maximum: self.receiver_limits.max_depth(),
            };
            capture_filter_error(&self.error_slot, error.clone());
            return Err(E::custom(error));
        }
        self.node_budget
            .try_consume(1)
            .map_err(filter_limit_error)
            .map_err(|error| {
                capture_filter_error(&self.error_slot, error.clone());
                E::custom(error)
            })?;
        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for FilterExpressionWireV1Seed<'a> {
    type Value = FilterExpressionWireV1;

    /// Decodes one expression map after charging its depth and node budget.
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.enter_node()?;
        deserializer.deserialize_map(ExpressionVisitor {
            receiver_limits: self.receiver_limits,
            node_budget: self.node_budget,
            depth: self.depth,
            error_slot: self.error_slot,
        })
    }
}
