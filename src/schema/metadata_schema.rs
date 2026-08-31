// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataSchema`] — schema validation for metadata and filters.

use std::collections::BTreeMap;
#[cfg(feature = "json")]
use std::io::Write;

#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeSession;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeSession;
use qubit_datatype::DataType;
#[cfg(feature = "json")]
use qubit_json::decode::JsonDecoder;
#[cfg(feature = "json")]
use qubit_json::encode::JsonEncoder;
use qubit_value::Value;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de;
#[cfg(feature = "json")]
use serde::de::Error as DeError;
use serde::ser::Error as SerError;

use crate::Metadata;
use crate::MetadataError;
use crate::MetadataResult;
use crate::MetadataValidationError;
use crate::MetadataValidationResult;
use crate::constants::STRICT_STRING_MAP_MAX_ENTRIES;
use crate::constants::STRICT_STRING_MAP_MAX_KEY_BYTES;
#[cfg(feature = "json")]
use crate::metadata_limits::MetadataLimits;
use crate::schema::MetadataField;
use crate::schema::MetadataSchemaBuilder;
use crate::schema::UnknownFilterFieldPolicy;
use crate::schema::UnknownMetadataFieldPolicy;
use crate::wire::METADATA_SCHEMA_WIRE_VERSION_V1;
use crate::wire::MetadataSchemaWireV1;
#[cfg(feature = "json")]
use crate::wire::MetadataSchemaWireV1Seed;
use crate::wire::StrictStringMap;
#[cfg(feature = "json")]
use crate::wire::StrictStringMapSeed;

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

    /// Decodes a strict metadata-schema JSON envelope using the metadata
    /// profile.
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
    /// Returns a structured budget error or `InvalidJson` for malformed strict
    /// schema input.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice(input: &[u8]) -> Result<Self, crate::MetadataWireDecodeError> {
        Self::decode_json_slice_with_limits(input, MetadataLimits::default())
    }

    /// Decodes a strict metadata-schema JSON envelope after applying
    /// `limits`.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete untrusted JSON input.
    /// * `limits` - Shared JSON limits for this decoding session.
    ///
    /// # Returns
    ///
    /// The decoded metadata schema.
    ///
    /// # Errors
    ///
    /// Returns a structured budget error or `InvalidJson` for syntax and strict
    /// schema-envelope failures.
    #[cfg(feature = "json")]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        limits: MetadataLimits,
    ) -> Result<Self, crate::MetadataWireDecodeError> {
        limits.validate().map_err(crate::MetadataWireDecodeError::InvalidJson)?;
        let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(limits.json_decode()));
        let wire = decoder
            .decode_seed_utf8(
                MetadataSchemaWireV1Seed::new(StrictStringMapSeed::new(
                    limits.max_schema_fields(),
                    limits.max_key_bytes(),
                )),
                input,
            )
            .map_err(Into::<crate::MetadataWireDecodeError>::into)?;
        if wire.version != METADATA_SCHEMA_WIRE_VERSION_V1 {
            return Err(crate::MetadataWireDecodeError::InvalidJson(
                <serde_json::Error as DeError>::custom("unsupported MetadataSchema wire format version"),
            ));
        }
        Ok(Self::new(
            wire.fields.into_inner(),
            wire.unknown_metadata_field_policy,
            wire.unknown_filter_field_policy,
        ))
    }

    /// Encodes this schema with the default JSON budget profile.
    #[cfg(feature = "json")]
    pub fn to_json_vec(&self) -> Result<Vec<u8>, crate::MetadataWireEncodeError> {
        self.to_json_vec_with_limits(crate::metadata_limits::default_json_encode_limits())
    }

    /// Encodes this schema with caller-provided JSON budgets.
    ///
    /// # Parameters
    ///
    /// * `limits` - Output and JSON-value budgets for this operation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::MetadataWireEncodeError`] when encoding exceeds a
    /// budget or serialization fails.
    #[cfg(feature = "json")]
    pub fn to_json_vec_with_limits(&self, limits: JsonEncodeLimits) -> Result<Vec<u8>, crate::MetadataWireEncodeError> {
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session).to_vec(self).map_err(Into::into)
    }

    /// Encodes this schema to a writer with the default JSON budget profile.
    #[cfg(feature = "json")]
    pub fn to_json_writer<W>(&self, writer: W) -> Result<(), crate::MetadataWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(writer, crate::metadata_limits::default_json_encode_limits())
    }

    /// Encodes this schema to a writer with caller-provided JSON budgets.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the compact JSON document.
    /// * `limits` - Output and JSON-value budgets for this operation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::MetadataWireEncodeError`] when encoding exceeds a
    /// budget, serialization fails, or `writer` rejects the output.
    #[cfg(feature = "json")]
    pub fn to_json_writer_with_limits<W>(
        &self,
        writer: W,
        limits: JsonEncodeLimits,
    ) -> Result<(), crate::MetadataWireEncodeError>
    where
        W: Write,
    {
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .write_buffered(writer, self)
            .map_err(Into::into)
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
            if field.is_required() && meta.get_raw(key).is_none_or(Value::is_unset) {
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
    pub(crate) fn validate_entry(&self, key: &str, value: &Value) -> MetadataResult<()> {
        match self.field(key) {
            Some(field) if field.is_required() && value.is_unset() => Err(MetadataError::MissingRequiredField {
                key: key.to_string(),
                expected: field.data_type(),
            }),
            Some(field) if field.data_type() != value.data_type() => {
                Err(MetadataError::type_mismatch(key, field.data_type(), value.data_type()))
            }
            Some(_) => Ok(()),
            None if matches!(self.unknown_metadata_field_policy, UnknownMetadataFieldPolicy::Reject) => {
                Err(MetadataError::UnknownField { key: key.to_string() })
            }
            None => Ok(()),
        }
    }

    /// Validates that this schema fits the strict V1 wire contract.
    ///
    /// This preflight checks the same field-count and key-byte limits enforced
    /// by [`Serialize::serialize`], allowing callers to reject invalid schemas
    /// before crossing a persistence or network boundary.
    ///
    /// # Returns
    ///
    /// `Ok(())` when every field can satisfy the schema map limits.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::WireLimitExceeded`] when the field count or a
    /// field key exceeds the strict V1 limit.
    #[inline]
    pub fn validate_wire_contract(&self) -> MetadataResult<()> {
        if self.fields.len() > STRICT_STRING_MAP_MAX_ENTRIES {
            return Err(MetadataError::WireLimitExceeded {
                kind: crate::MetadataWireLimitKind::Entries,
                value: self.fields.len(),
                maximum: STRICT_STRING_MAP_MAX_ENTRIES,
            });
        }
        if let Some(key) = self
            .fields
            .keys()
            .find(|key| key.len() > STRICT_STRING_MAP_MAX_KEY_BYTES)
        {
            return Err(MetadataError::WireLimitExceeded {
                kind: crate::MetadataWireLimitKind::KeyBytes,
                value: key.len(),
                maximum: STRICT_STRING_MAP_MAX_KEY_BYTES,
            });
        }
        Ok(())
    }
}

impl Serialize for MetadataSchema {
    /// Serializes this schema as its strict v1 envelope.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.validate_wire_contract().map_err(<S::Error as SerError>::custom)?;
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
            return Err(de::Error::custom("unsupported MetadataSchema wire format version"));
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
