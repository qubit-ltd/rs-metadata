// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Incremental V1 filter-expression decoding with receiver limits.

use std::fmt;

use qubit_budget::{
    BudgetError,
    ResourceBudget,
};
use qubit_value::ValueWirePayloadV1;
use serde::Deserialize;
use serde::de::{
    self,
    DeserializeSeed,
    Error as _,
    MapAccess,
    SeqAccess,
    Visitor,
};

use super::FilterExpressionWireV1;
use crate::{
    FilterLimitKind,
    FilterLimits,
    MetadataError,
};

/// Expression variant tag in the V1 wire representation.
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExpressionKind {
    /// Constant true expression.
    All,
    /// Constant false expression.
    None,
    /// Equality condition.
    Eq,
    /// Inequality condition.
    Ne,
    /// Less-than condition.
    Lt,
    /// Less-or-equal condition.
    Le,
    /// Greater-than condition.
    Gt,
    /// Greater-or-equal condition.
    Ge,
    /// Inclusion condition.
    In,
    /// Exclusion condition.
    NotIn,
    /// Existence condition.
    Exists,
    /// Non-existence condition.
    NotExists,
    /// Logical AND.
    And,
    /// Logical OR.
    Or,
    /// Logical negation.
    Not,
}

/// Seed that decodes one expression while charging filter-domain budgets.
pub(crate) struct FilterExpressionWireV1Seed<'a> {
    receiver_limits: FilterLimits,
    node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
    depth: usize,
}

impl<'a> FilterExpressionWireV1Seed<'a> {
    /// Creates a seed for one expression at `depth`.
    pub(crate) const fn new(
        receiver_limits: FilterLimits,
        node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
        depth: usize,
    ) -> Self {
        Self {
            receiver_limits,
            node_budget,
            depth,
        }
    }

    /// Charges one expression node before reading its map body.
    fn enter_node<E>(&mut self) -> Result<(), E>
    where
        E: de::Error,
    {
        if self.depth > self.receiver_limits.max_depth() {
            return Err(E::custom(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::Depth,
                value: self.depth,
                maximum: self.receiver_limits.max_depth(),
            }));
        }
        self.node_budget
            .try_consume(1)
            .map_err(filter_limit_error)
            .map_err(E::custom)?;
        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for FilterExpressionWireV1Seed<'a> {
    type Value = FilterExpressionWireV1;

    /// Decodes one expression map after charging its depth and node budget.
    fn deserialize<D>(
        mut self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.enter_node()?;
        deserializer.deserialize_map(ExpressionVisitor {
            receiver_limits: self.receiver_limits,
            node_budget: self.node_budget,
            depth: self.depth,
        })
    }
}

/// Converts shared budget facts to the established metadata filter error.
fn filter_limit_error(
    error: BudgetError<FilterLimitKind, usize>,
) -> MetadataError {
    match error {
        BudgetError::Insufficient {
            resource,
            limit,
            remaining,
            requested,
        } => MetadataError::FilterLimitExceeded {
            kind: resource,
            value: limit.saturating_sub(remaining).saturating_add(requested),
            maximum: limit,
        },
        BudgetError::LimitExceeded { .. } => {
            unreachable!(
                "filter node budgets only report insufficient capacity"
            )
        }
    }
}

/// Temporarily owned fields collected from one expression map.
struct ExpressionFields {
    kind: Option<ExpressionKind>,
    key: Option<String>,
    value: Option<ValueWirePayloadV1>,
    values: Option<Vec<ValueWirePayloadV1>>,
    children: Option<Vec<FilterExpressionWireV1>>,
    expression: Option<Box<FilterExpressionWireV1>>,
}

impl ExpressionFields {
    /// Converts validated fields into the corresponding V1 wire variant.
    fn into_wire(self) -> Result<FilterExpressionWireV1, String> {
        let Self {
            kind,
            key,
            value,
            values,
            children,
            expression,
        } = self;
        let kind = kind.ok_or_else(|| "missing field `kind`".to_owned())?;
        match kind {
            ExpressionKind::All => {
                ensure_absent(key, "key")?;
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::All)
            }
            ExpressionKind::None => {
                ensure_absent(key, "key")?;
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::None)
            }
            ExpressionKind::Eq => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Eq {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::Ne => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Ne {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::Lt => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Lt {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::Le => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Le {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::Gt => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Gt {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::Ge => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Ge {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::In => {
                ensure_absent(value, "value")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::In {
                    key: required(key, "key")?,
                    values: required(values, "values")?,
                })
            }
            ExpressionKind::NotIn => {
                ensure_absent(value, "value")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::NotIn {
                    key: required(key, "key")?,
                    values: required(values, "values")?,
                })
            }
            ExpressionKind::Exists => {
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Exists {
                    key: required(key, "key")?,
                })
            }
            ExpressionKind::NotExists => {
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::NotExists {
                    key: required(key, "key")?,
                })
            }
            ExpressionKind::And => {
                ensure_absent(key, "key")?;
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::And {
                    children: required(children, "children")?,
                })
            }
            ExpressionKind::Or => {
                ensure_absent(key, "key")?;
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Or {
                    children: required(children, "children")?,
                })
            }
            ExpressionKind::Not => {
                ensure_absent(key, "key")?;
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                Ok(FilterExpressionWireV1::Not {
                    expression: required(expression, "expression")?,
                })
            }
        }
    }
}

/// Returns a required field or a missing-field message.
fn required<T>(value: Option<T>, field: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("missing field `{field}`"))
}

/// Rejects a field that is not valid for the selected expression kind.
fn ensure_absent<T>(value: Option<T>, field: &str) -> Result<(), String> {
    if value.is_some() {
        Err(format!("unknown field `{field}` for this expression kind"))
    } else {
        Ok(())
    }
}

/// Checks one decoded key against receiver filter limits.
fn check_key<E>(key: &str, receiver_limits: FilterLimits) -> Result<(), E>
where
    E: de::Error,
{
    if key.len() > receiver_limits.max_key_bytes() {
        return Err(E::custom(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::KeyBytes,
            value: key.len(),
            maximum: receiver_limits.max_key_bytes(),
        }));
    }
    Ok(())
}

/// Visitor for one expression map.
struct ExpressionVisitor<'a> {
    receiver_limits: FilterLimits,
    node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
    depth: usize,
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
                    check_key::<A::Error>(&key, self.receiver_limits)?;
                    fields.key = Some(key);
                }
                "value" => {
                    if fields.value.is_some() {
                        return Err(de::Error::duplicate_field("value"));
                    }
                    let value = map.next_value::<ValueWirePayloadV1>()?;
                    fields.value = Some(value);
                }
                "values" => {
                    if fields.values.is_some() {
                        return Err(de::Error::duplicate_field("values"));
                    }
                    fields.values = Some(map.next_value_seed(
                        ValueSequenceSeed::new(self.receiver_limits),
                    )?);
                }
                "children" => {
                    if fields.children.is_some() {
                        return Err(de::Error::duplicate_field("children"));
                    }
                    fields.children = Some(map.next_value_seed(
                        ExpressionSequenceSeed::new(
                            self.receiver_limits,
                            &mut *self.node_budget,
                            self.depth.saturating_add(1),
                        ),
                    )?);
                }
                "expression" => {
                    if fields.expression.is_some() {
                        return Err(de::Error::duplicate_field("expression"));
                    }
                    fields.expression = Some(Box::new(map.next_value_seed(
                        FilterExpressionWireV1Seed::new(
                            self.receiver_limits,
                            &mut *self.node_budget,
                            self.depth.saturating_add(1),
                        ),
                    )?));
                }
                _ => {
                    return Err(de::Error::unknown_field(
                        &field,
                        &[
                            "kind",
                            "key",
                            "value",
                            "values",
                            "children",
                            "expression",
                        ],
                    ));
                }
            }
        }
        fields.into_wire().map_err(de::Error::custom)
    }
}

/// Seed that bounds a sequence of child expressions.
struct ExpressionSequenceSeed<'a> {
    receiver_limits: FilterLimits,
    node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
    depth: usize,
}

impl<'a> ExpressionSequenceSeed<'a> {
    /// Creates a bounded child-expression sequence seed.
    const fn new(
        receiver_limits: FilterLimits,
        node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
        depth: usize,
    ) -> Self {
        Self {
            receiver_limits,
            node_budget,
            depth,
        }
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExpressionSequenceSeed<'a> {
    type Value = Vec<FilterExpressionWireV1>;

    /// Decodes child expressions with bounded growth.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExpressionSequenceVisitor {
            receiver_limits: self.receiver_limits,
            node_budget: self.node_budget,
            depth: self.depth,
        })
    }
}

/// Visitor for a bounded child-expression sequence.
struct ExpressionSequenceVisitor<'a> {
    receiver_limits: FilterLimits,
    node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
    depth: usize,
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
        while let Some(child) =
            sequence.next_element_seed(ExpressionElementSeed {
                receiver_limits: self.receiver_limits,
                node_budget: &mut *self.node_budget,
                depth: self.depth,
            })?
        {
            children.push(child);
        }
        Ok(children)
    }
}

/// Seed that checks one child collection item before reading its body.
struct ExpressionElementSeed<'a> {
    receiver_limits: FilterLimits,
    node_budget: &'a mut ResourceBudget<FilterLimitKind, usize>,
    depth: usize,
}

impl<'de, 'a> DeserializeSeed<'de> for ExpressionElementSeed<'a> {
    type Value = FilterExpressionWireV1;

    /// Charges one child item before delegating to the recursive expression
    /// seed.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        FilterExpressionWireV1Seed::new(
            self.receiver_limits,
            self.node_budget,
            self.depth,
        )
        .deserialize(deserializer)
    }
}

/// Seed that bounds a membership-value sequence.
struct ValueSequenceSeed {
    receiver_limits: FilterLimits,
}

impl ValueSequenceSeed {
    /// Creates a bounded membership-value sequence seed.
    const fn new(receiver_limits: FilterLimits) -> Self {
        Self { receiver_limits }
    }
}

impl<'de> DeserializeSeed<'de> for ValueSequenceSeed {
    type Value = Vec<ValueWirePayloadV1>;

    /// Decodes membership values with receiver-controlled bounds.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ValueSequenceVisitor {
            receiver_limits: self.receiver_limits,
        })
    }
}

/// Visitor for a bounded membership-value sequence.
struct ValueSequenceVisitor {
    receiver_limits: FilterLimits,
}

impl<'de> Visitor<'de> for ValueSequenceVisitor {
    type Value = Vec<ValueWirePayloadV1>;

    /// Describes a sequence of scalar V1 payloads.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a sequence of scalar V1 value payloads")
    }

    /// Decodes membership values one by one and bounds vector growth.
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let maximum = self.receiver_limits.max_set_values();
        let capacity = sequence.size_hint().unwrap_or(0).min(maximum);
        let mut values = Vec::with_capacity(capacity);
        while let Some(value) =
            sequence.next_element_seed(ValueElementSeed {
                receiver_limits: self.receiver_limits,
                next_len: values.len().saturating_add(1),
            })?
        {
            values.push(value);
        }
        Ok(values)
    }
}

/// Seed that checks one membership item before reading its payload.
struct ValueElementSeed {
    receiver_limits: FilterLimits,
    next_len: usize,
}

impl<'de> DeserializeSeed<'de> for ValueElementSeed {
    type Value = ValueWirePayloadV1;

    /// Decodes one membership payload after charging its collection position.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if self.next_len > self.receiver_limits.max_set_values() {
            return Err(D::Error::custom(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::SetValues,
                value: self.next_len,
                maximum: self.receiver_limits.max_set_values(),
            }));
        }
        ValueWirePayloadV1::deserialize(deserializer)
    }
}
