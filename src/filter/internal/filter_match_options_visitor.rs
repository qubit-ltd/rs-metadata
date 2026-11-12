// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Object-only deserialization for filter match options.

use std::fmt;

use qubit_datatype::NumericComparisonPolicy;
use serde::Deserializer;
use serde::de::MapAccess;
use serde::de::Visitor;

use crate::FilterMatchOptions;

/// Field names accepted by the filter match options object visitor.
pub(super) const FIELD_NAMES: &[&str] = &["numeric_comparison_policy"];

/// Visits the object representation of filter match options.
pub(super) struct FilterMatchOptionsVisitor;

impl<'de> Visitor<'de> for FilterMatchOptionsVisitor {
    /// The options value produced by this visitor.
    type Value = FilterMatchOptions;

    /// Describes the required wire shape in deserializer diagnostics.
    ///
    /// # Parameters
    ///
    /// * `formatter` - The destination for the human-readable description.
    ///
    /// # Returns
    ///
    /// The result of writing the description.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a filter match options object")
    }

    /// Reads exactly one numeric comparison policy.
    ///
    /// # Type Parameters
    ///
    /// * `A` - The map access implementation supplied by Serde.
    ///
    /// # Parameters
    ///
    /// * `map` - The object fields being decoded.
    ///
    /// # Returns
    ///
    /// The options containing the decoded policy.
    ///
    /// # Errors
    ///
    /// Returns an error for missing, duplicate, unknown, or invalid fields.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut numeric_comparison_policy = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "numeric_comparison_policy" => {
                    if numeric_comparison_policy.is_some() {
                        return Err(serde::de::Error::duplicate_field(
                            "numeric_comparison_policy",
                        ));
                    }
                    numeric_comparison_policy = Some(map.next_value::<NumericComparisonPolicy>()?);
                }
                _ => return Err(serde::de::Error::unknown_field(&key, FIELD_NAMES)),
            }
        }

        let policy = numeric_comparison_policy
            .ok_or_else(|| serde::de::Error::missing_field("numeric_comparison_policy"))?;
        Ok(FilterMatchOptions::from_numeric_comparison_policy(policy))
    }
}

/// Deserializes the strict object representation of filter match options.
///
/// # Type Parameters
///
/// * `D` - The deserializer supplying the options representation.
///
/// # Parameters
///
/// * `deserializer` - The source representation to decode.
///
/// # Returns
///
/// The decoded filter match options.
///
/// # Errors
///
/// Returns an error if the representation is not an object with exactly one
/// valid numeric comparison policy.
pub(crate) fn deserialize_filter_match_options<'de, D>(
    deserializer: D,
) -> Result<FilterMatchOptions, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_struct("FilterMatchOptions", FIELD_NAMES, FilterMatchOptionsVisitor)
}
