/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! [`MetadataField`] — one field definition in a metadata schema.

use qubit_common::DataType;
use serde::{Deserialize, Serialize};

/// Definition of one metadata field in a [`crate::MetadataSchema`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataField {
    /// Runtime data type of this field.
    data_type: DataType,
    /// Whether this field must be present when validating metadata.
    required: bool,
}

impl MetadataField {
    /// Creates a field definition.
    #[inline]
    #[must_use]
    pub fn new(data_type: DataType, required: bool) -> Self {
        Self {
            data_type,
            required,
        }
    }

    /// Returns the runtime data type of this field.
    #[inline]
    #[must_use]
    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    /// Returns `true` when this field is required.
    #[inline]
    #[must_use]
    pub fn is_required(&self) -> bool {
        self.required
    }
}
