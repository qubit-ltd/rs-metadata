// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Map visitor for one filter expression node.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use qubit_budget::ResourceBudget;
use qubit_value::ValueWirePayloadV1Seed;
use serde::de;
use serde::de::MapAccess;
use serde::de::Visitor;

use super::super::filter_expression_wire_v1::FilterExpressionWireV1;
use super::super::filter_expression_wire_v1_seed::FilterExpressionWireV1Seed;
use super::expression_fields::ExpressionFields;
use super::expression_sequence_seed::ExpressionSequenceSeed;
use super::limit_support::check_key;
use super::value_sequence_seed::ValueSequenceSeed;
use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::MetadataError;

/// Visitor for one expression map.
pub(in crate::filter::wire) struct ExpressionVisitor<'a> {
    /// Receiver-side AST limits.
    pub(in crate::filter::wire) receiver_limits: FilterLimits,
    /// Remaining node budget.
    pub(in crate::filter::wire) node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
    /// Current expression depth.
    pub(in crate::filter::wire) depth: usize,
    /// First structured limit error.
    pub(in crate::filter::wire) error_slot: Rc<RefCell<Option<MetadataError>>>,
}

impl<'de, 'a> Visitor<'de> for ExpressionVisitor<'a> {
    type Value = FilterExpressionWireV1;

    /// Describes one strict filter expression node.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a strict metadata-filter expression object")
    }

    /// Decodes fields in any order while rejecting duplicates and unknowns.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut fields = ExpressionFields {
            kind: None,
            key: None,
            value: None,
            values: None,
            children: None,
            expression: None,
        };
        while let Some(field) = map.next_key::<String>()? {
            match field.as_str() {
                "kind" => {
                    if fields.kind.is_some() {
                        return Err(de::Error::duplicate_field("kind"));
                    }
                    fields.kind = Some(map.next_value()?);
                }
                "key" => {
                    if fields.key.is_some() {
                        return Err(de::Error::duplicate_field("key"));
                    }
                    let key = map.next_value::<String>()?;
                    check_key::<A::Error>(&key, self.receiver_limits, &self.error_slot)?;
                    fields.key = Some(key);
                }
                "value" => {
                    if fields.value.is_some() {
                        return Err(de::Error::duplicate_field("value"));
                    }
                    let value = map.next_value_seed(ValueWirePayloadV1Seed::new())?;
                    fields.value = Some(value);
                }
                "values" => {
                    if fields.values.is_some() {
                        return Err(de::Error::duplicate_field("values"));
                    }
                    fields.values = Some(map.next_value_seed(ValueSequenceSeed {
                        receiver_limits: self.receiver_limits,
                        error_slot: Rc::clone(&self.error_slot),
                    })?);
                }
                "children" => {
                    if fields.children.is_some() {
                        return Err(de::Error::duplicate_field("children"));
                    }
                    fields.children = Some(map.next_value_seed(ExpressionSequenceSeed {
                        receiver_limits: self.receiver_limits,
                        node_budget: &mut *self.node_budget,
                        depth: self.depth.saturating_add(1),
                        error_slot: Rc::clone(&self.error_slot),
                    })?);
                }
                "expression" => {
                    if fields.expression.is_some() {
                        return Err(de::Error::duplicate_field("expression"));
                    }
                    fields.expression = Some(Box::new(map.next_value_seed(FilterExpressionWireV1Seed::new(
                        self.receiver_limits,
                        &mut *self.node_budget,
                        self.depth.saturating_add(1),
                        Rc::clone(&self.error_slot),
                    ))?));
                }
                _ => {
                    return Err(de::Error::unknown_field(
                        &field,
                        &["kind", "key", "value", "values", "children", "expression"],
                    ));
                }
            }
        }
        fields.into_wire().map_err(de::Error::custom)
    }
}
