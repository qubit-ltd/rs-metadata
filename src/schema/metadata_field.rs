// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataField`] — one field definition in a metadata schema.

use qubit_datatype::DataType;
use serde::{
    Deserialize,
    Serialize,
};

/// Definition of one metadata field in a [`crate::MetadataSchema`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataField {
    /// Runtime data type of this field.
    data_type: DataType,
    /// Whether this field must be present when validating metadata.
    required: bool,
}

impl MetadataField {
    /// Creates a field definition.
    ///
    /// # Parameters
    ///
    /// * `data_type` - Concrete data type accepted by the field.
    /// * `required` - Whether metadata must provide a concrete value.
    ///
    /// # Returns
    ///
    /// A new field definition.
    #[inline]
    #[must_use = "the constructed metadata field should be used"]
    pub fn new(data_type: DataType, required: bool) -> Self {
        Self {
            data_type,
            required,
        }
    }

    /// Returns the runtime data type of this field.
    ///
    /// # Returns
    ///
    /// The declared field data type.
    #[inline(always)]
    #[must_use = "the metadata field type should be inspected"]
    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    /// Returns `true` when this field is required.
    ///
    /// # Returns
    ///
    /// `true` when validation requires a concrete value for this field.
    #[inline(always)]
    #[must_use]
    pub fn is_required(&self) -> bool {
        self.required
    }
}
