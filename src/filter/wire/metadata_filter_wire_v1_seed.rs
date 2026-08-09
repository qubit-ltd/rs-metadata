// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Incremental V1 metadata-filter envelope decoding.

use std::fmt;

use qubit_budget::ResourceBudget;
use qubit_budget::ResourceLimit;
use qubit_value::WireBudget;
use serde::de::{
    self,
    DeserializeSeed,
    MapAccess,
    Visitor,
};

use super::{
    FilterExpressionWireV1Seed,
    MetadataFilterWireV1,
};
use crate::{
    FilterLimitKind,
    FilterLimits,
    FilterMatchOptions,
};

/// Seed that decodes one strict metadata-filter envelope.
pub(crate) struct MetadataFilterWireV1Seed<'a> {
    receiver_limits: FilterLimits,
    budget: &'a mut WireBudget,
}

impl<'a> MetadataFilterWireV1Seed<'a> {
    /// Creates a bounded filter-envelope seed.
    pub(crate) const fn new(
        receiver_limits: FilterLimits,
        budget: &'a mut WireBudget,
    ) -> Self {
        Self {
            receiver_limits,
            budget,
        }
    }
}

impl<'de, 'a> DeserializeSeed<'de> for MetadataFilterWireV1Seed<'a> {
    type Value = MetadataFilterWireV1;

    /// Decodes the strict envelope and delegates expression decoding to a seed.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(MetadataFilterWireVisitor {
            receiver_limits: self.receiver_limits,
            budget: self.budget,
        })
    }
}

/// Visitor for the strict metadata-filter envelope.
struct MetadataFilterWireVisitor<'a> {
    receiver_limits: FilterLimits,
    budget: &'a mut WireBudget,
}

impl<'de, 'a> Visitor<'de> for MetadataFilterWireVisitor<'a> {
    type Value = MetadataFilterWireV1;

    /// Describes the strict metadata-filter envelope.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a strict metadata-filter V1 envelope")
    }

    /// Decodes envelope fields in any order and rejects duplicates or unknowns.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut version = None;
        let mut expression = None;
        let mut options = None;
        let mut node_budget = ResourceBudget::new(
            FilterLimitKind::Nodes,
            ResourceLimit::new(self.receiver_limits.max_nodes() as u64),
        );
        while let Some(field) = map.next_key::<String>()? {
            match field.as_str() {
                "version" => {
                    if version.is_some() {
                        return Err(de::Error::duplicate_field("version"));
                    }
                    version = Some(map.next_value::<u8>()?);
                }
                "expression" => {
                    if expression.is_some() {
                        return Err(de::Error::duplicate_field("expression"));
                    }
                    expression = Some(map.next_value_seed(
                        FilterExpressionWireV1Seed::new(
                            self.receiver_limits,
                            &mut node_budget,
                            &mut *self.budget,
                            1,
                        ),
                    )?);
                }
                "options" => {
                    if options.is_some() {
                        return Err(de::Error::duplicate_field("options"));
                    }
                    options = Some(map.next_value::<FilterMatchOptions>()?);
                }
                _ => {
                    return Err(de::Error::unknown_field(
                        &field,
                        &["version", "expression", "options"],
                    ));
                }
            }
        }
        Ok(MetadataFilterWireV1::new(
            version.ok_or_else(|| de::Error::missing_field("version"))?,
            expression.ok_or_else(|| de::Error::missing_field("expression"))?,
            options.ok_or_else(|| de::Error::missing_field("options"))?,
        ))
    }
}
