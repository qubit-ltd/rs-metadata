// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataSchema`] — schema validation for metadata and filters.

use std::collections::BTreeMap;

use qubit_datatype::DataType;
use qubit_value::Value;
#[cfg(feature = "json")]
use qubit_value::WireBudget;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::{
        self,
    },
};
#[cfg(feature = "json")]
use serde::de::DeserializeSeed;

use crate::schema::{
    MetadataField,
    MetadataSchemaBuilder,
    UnknownFilterFieldPolicy,
    UnknownMetadataFieldPolicy,
};
use crate::wire::{
    METADATA_SCHEMA_WIRE_VERSION_V1,
    MetadataSchemaWireV1,
    StrictStringMap,
};
#[cfg(feature = "json")]
use crate::wire::{MetadataSchemaWireV1Seed, StrictStringMapSeed};
use crate::{
    Metadata,
    MetadataError,
    MetadataResult,
    MetadataValidationError,
    MetadataValidationResult,
};

/// Schema for metadata fields.
///
/// A schema declares valid keys, their concrete [`DataType`], and whether they
/// are required. It can validate actual [`Metadata`] values and validate that a
/// [`crate::MetadataFilter`] references known fields with compatible operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataSchema {
    /// Field definitions keyed by metadata key.
    fields: BTreeMap<String, MetadataField>,
    /// How validation handles unknown metadata keys.
    unknown_metadata_field_policy: UnknownMetadataFieldPolicy,
    /// How validation handles unknown filter keys.
    unknown_filter_field_policy: UnknownFilterFieldPolicy,
}

impl MetadataSchema {
    /// Creates a schema builder.
    ///
    /// # Returns
    ///
    /// An empty schema builder using the default unknown-field policy.
    #[inline(always)]
    #[must_use]
    pub fn builder() -> MetadataSchemaBuilder {
        MetadataSchemaBuilder::default()
    }

    /// Decodes a strict metadata-schema JSON envelope using the default byte
    /// limit.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete untrusted JSON input.
    ///
    /// # Returns
    ///
    /// The decoded metadata schema.
    ///
    /// # Errors
    ///
    /// Returns an input-size error before parsing or an invalid-JSON error for
    /// malformed strict schema input.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice(
        input: &[u8],
    ) -> Result<Self, crate::MetadataWireDecodeError> {
        Self::decode_json_slice_with_limits(
            input,
            crate::MetadataWireLimits::default(),
        )
    }

    /// Decodes a strict metadata-schema JSON envelope after applying
    /// `wire_limits`.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete untrusted JSON input.
    /// * `wire_limits` - Byte limit checked before JSON parsing.
    ///
    /// # Returns
    ///
    /// The decoded metadata schema.
    ///
    /// # Errors
    ///
    /// Returns an input-size error before parsing or an invalid-JSON error for
    /// syntax or strict schema-envelope failures.
    #[cfg(feature = "json")]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        wire_limits: crate::MetadataWireLimits,
    ) -> Result<Self, crate::MetadataWireDecodeError> {
        let mut budget = wire_limits.wire().begin(input.len())?;
        let mut deserializer = serde_json::Deserializer::from_slice(input);
        let wire = MetadataSchemaWireV1Seed::new(StrictStringMapSeed::new(
            wire_limits.max_schema_fields(),
            wire_limits.max_key_bytes(),
        ))
        .deserialize(&mut deserializer)
        .map_err(|error| {
            crate::MetadataWireDecodeError::from_strict_map_error(
                error,
                crate::MetadataWireLimitKind::SchemaFields,
            )
        })?;
        deserializer
            .end()
            .map_err(crate::MetadataWireDecodeError::InvalidJson)?;
        if wire.version != METADATA_SCHEMA_WIRE_VERSION_V1 {
            return Err(crate::MetadataWireDecodeError::InvalidJson(
                <serde_json::Error as serde::de::Error>::custom(
                    "unsupported MetadataSchema wire format version",
                ),
            ));
        }
        let schema = Self::new(
            wire.fields.into_inner(),
            wire.unknown_metadata_field_policy,
            wire.unknown_filter_field_policy,
        );
        schema.validate_wire_limits(wire_limits, &mut budget)?;
        Ok(schema)
    }

    /// Creates a schema from field definitions and unknown-field policies.
    ///
    /// # Parameters
    ///
    /// * `fields` - Field definitions keyed by metadata key.
    /// * `unknown_metadata_field_policy` - Policy for undeclared metadata keys.
    /// * `unknown_filter_field_policy` - Policy for undeclared filter keys.
    ///
    /// # Returns
    ///
    /// A new immutable schema.
    #[inline]
    pub(crate) fn new(
        fields: BTreeMap<String, MetadataField>,
        unknown_metadata_field_policy: UnknownMetadataFieldPolicy,
        unknown_filter_field_policy: UnknownFilterFieldPolicy,
    ) -> Self {
        Self {
            fields,
            unknown_metadata_field_policy,
            unknown_filter_field_policy,
        }
    }

    /// Returns the field definition for `key`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to look up.
    ///
    /// # Returns
    ///
    /// `Some` field definition for a declared key; otherwise, `None`.
    #[inline(always)]
    pub fn field(&self, key: &str) -> Option<&MetadataField> {
        self.fields.get(key)
    }

    /// Returns the declared data type for `key`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to look up.
    ///
    /// # Returns
    ///
    /// `Some` declared data type for a known key; otherwise, `None`.
    #[inline(always)]
    pub fn field_type(&self, key: &str) -> Option<DataType> {
        self.field(key).map(MetadataField::data_type)
    }

    /// Returns the policy for undeclared metadata fields.
    ///
    /// # Returns
    ///
    /// The policy applied to undeclared metadata keys.
    #[inline(always)]
    #[must_use]
    pub fn unknown_metadata_field_policy(&self) -> UnknownMetadataFieldPolicy {
        self.unknown_metadata_field_policy
    }

    /// Returns the policy for undeclared filter fields.
    ///
    /// # Returns
    ///
    /// The policy applied to filter keys not declared in this schema.
    #[inline(always)]
    #[must_use]
    pub fn unknown_filter_field_policy(&self) -> UnknownFilterFieldPolicy {
        self.unknown_filter_field_policy
    }

    /// Returns an iterator over schema fields in key-sorted order.
    ///
    /// # Returns
    ///
    /// An iterator yielding key and field-definition pairs.
    #[inline(always)]
    pub fn fields(&self) -> impl Iterator<Item = (&str, &MetadataField)> {
        self.fields.iter().map(|(key, field)| (key.as_str(), field))
    }

    /// Validates a metadata object against this schema.
    ///
    /// # Parameters
    ///
    /// * `meta` - Metadata object to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` when every entry satisfies the schema.
    ///
    /// # Errors
    ///
    /// Returns an aggregate error containing every required-field,
    /// declared-type, and unknown-field issue discovered during this
    /// validation pass.
    pub fn validate(&self, meta: &Metadata) -> MetadataValidationResult<()> {
        let mut issues = Vec::new();
        for (key, field) in &self.fields {
            if field.is_required()
                && meta.get_raw(key).is_none_or(Value::is_unset)
            {
                issues.push(MetadataError::MissingRequiredField {
                    key: key.clone(),
                    expected: field.data_type(),
                });
            }
        }

        for (key, value) in meta.iter() {
            if self
                .field(key)
                .is_some_and(|field| field.is_required() && value.is_unset())
            {
                continue;
            }
            if let Err(error) = self.validate_entry(key, value) {
                issues.push(error);
            }
        }
        if let Some(error) = MetadataValidationError::from_issues(issues) {
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Validates one metadata entry against this schema.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to validate.
    /// * `value` - Stored value to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the entry is accepted.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::UnknownField`] for a rejected undeclared key,
    /// or [`MetadataError::TypeMismatch`] for a declared type mismatch.
    pub(crate) fn validate_entry(
        &self,
        key: &str,
        value: &Value,
    ) -> MetadataResult<()> {
        match self.field(key) {
            Some(field) if field.is_required() && value.is_unset() => {
                Err(MetadataError::MissingRequiredField {
                    key: key.to_string(),
                    expected: field.data_type(),
                })
            }
            Some(field) if field.data_type() != value.data_type() => {
                Err(MetadataError::type_mismatch(
                    key,
                    field.data_type(),
                    value.data_type(),
                ))
            }
            Some(_) => Ok(()),
            None if matches!(
                self.unknown_metadata_field_policy,
                UnknownMetadataFieldPolicy::Reject
            ) =>
            {
                Err(MetadataError::UnknownField {
                    key: key.to_string(),
                })
            }
            None => Ok(()),
        }
    }

    /// Validates decoded schema resources against JSON wire limits.
    ///
    /// # Parameters
    ///
    /// * `wire_limits` - Bounds applied to the decoded schema.
    ///
    /// # Errors
    ///
    /// Returns a wire-limit error when the schema has too many fields or a
    /// field name exceeds the configured UTF-8 byte bound.
    #[cfg(feature = "json")]
    fn validate_wire_limits(
        &self,
        wire_limits: crate::MetadataWireLimits,
        budget: &mut WireBudget,
    ) -> Result<(), crate::MetadataWireDecodeError> {
        wire_limits.check_schema_fields(self.fields.len())?;
        budget.check_node()?;
        budget.check_map_entries(self.fields.len())?;
        self.fields.keys().try_for_each(|key| {
            wire_limits.check_key_bytes(key.len())?;
            budget.check_string_bytes(key.len())?;
            Ok(())
        })
    }
}

impl Serialize for MetadataSchema {
    /// Serializes this schema as its strict v1 envelope.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        MetadataSchemaWireV1 {
            version: METADATA_SCHEMA_WIRE_VERSION_V1,
            fields: &self.fields,
            unknown_metadata_field_policy: self.unknown_metadata_field_policy,
            unknown_filter_field_policy: self.unknown_filter_field_policy,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MetadataSchema {
    /// Deserializes only the strict v1 schema envelope.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire: MetadataSchemaWireV1<StrictStringMap<MetadataField>> =
            MetadataSchemaWireV1::deserialize(deserializer)?;
        if wire.version != METADATA_SCHEMA_WIRE_VERSION_V1 {
            return Err(de::Error::custom(
                "unsupported MetadataSchema wire format version",
            ));
        }
        Ok(Self::new(
            wire.fields.into_inner(),
            wire.unknown_metadata_field_policy,
            wire.unknown_filter_field_policy,
        ))
    }
}

impl Default for MetadataSchema {
    #[inline]
    fn default() -> Self {
        Self {
            fields: BTreeMap::new(),
            unknown_metadata_field_policy: UnknownMetadataFieldPolicy::Reject,
            unknown_filter_field_policy: UnknownFilterFieldPolicy::Reject,
        }
    }
}
