// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provides the [`Metadata`] type — a structured, ordered, typed key-value
//! store.

use std::collections::BTreeMap;
use std::fmt;
#[cfg(feature = "json")]
use std::io::Write;

#[cfg(feature = "json")]
use qubit_budget::MeasuredBudgetError;
#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeSession;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeSession;
#[cfg(feature = "json")]
use qubit_budget::json::JsonResource;
use qubit_datatype::DataConversionTarget;
use qubit_datatype::DataType;
#[cfg(feature = "json")]
use qubit_json::text::JsonDecodeError;
#[cfg(feature = "json")]
use qubit_json::text::decode_slice_seed;
#[cfg(feature = "json")]
use qubit_json::text::encode_to_vec;
#[cfg(feature = "json")]
use qubit_json::text::encode_to_writer;
use qubit_redact::Redact;
use qubit_redact::RedactedKeyedResult;
use qubit_redact::RedactionSession;
use qubit_value::Value;
use qubit_value::ValueRef;
use qubit_value::ValueWirePayloadV1;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de;
#[cfg(feature = "json")]
use serde::de::Error as DeError;
use serde::ser::Error as SerError;

use crate::MetadataError;
use crate::MetadataResult;
#[cfg(feature = "schema")]
use crate::MetadataSchema;
use crate::constants::STRICT_STRING_MAP_MAX_ENTRIES;
use crate::constants::STRICT_STRING_MAP_MAX_KEY_BYTES;
#[cfg(feature = "json")]
use crate::metadata_limits::MetadataLimits;
use crate::wire::METADATA_WIRE_VERSION_V1;
use crate::wire::MetadataWireV1;
#[cfg(feature = "json")]
use crate::wire::MetadataWireV1Seed;
use crate::wire::MetadataWireValuesRef;
use crate::wire::StrictStringMap;
#[cfg(feature = "json")]
use crate::wire::StrictStringMapSeed;

/// A structured, ordered, typed key-value store for metadata fields.
///
/// `Metadata` stores values as [`qubit_value::Value`], preserving concrete Rust
/// scalar types such as `i64`, `u32`, `f64`, `String`, and `bool`.  This avoids
/// the ambiguity of a single JSON number type while still allowing callers to
/// store explicit `Value::Json` values when they really need JSON payloads.
/// [`Value::Unset`] retains a declared type but represents no concrete metadata
/// value: typed reads report [`MetadataError::MissingValue`]. When the optional
/// `schema` or `filter` features are enabled, their validation and matching
/// APIs treat it as a missing concrete value.
///
/// Use [`Metadata::with`] for fluent construction and [`Metadata::set`] when
/// mutating an existing object. The typed [`Metadata::get`] and
/// [`Metadata::try_get`] accessors convert stored values through
/// [`qubit_datatype::DataConversionTarget`]; use [`Metadata::get_raw`] when
/// the stored runtime [`qubit_value::Value`] must be inspected without
/// conversion.
#[derive(Clone, PartialEq, Default)]
pub struct Metadata(
    /// Stored values indexed by metadata key.
    BTreeMap<String, Value>,
);

impl Metadata {
    /// Creates an empty metadata object.
    ///
    /// # Returns
    ///
    /// An empty metadata object.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Decodes a strict metadata JSON envelope using the metadata profile.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete untrusted JSON input.
    ///
    /// # Returns
    ///
    /// The decoded metadata object.
    ///
    /// # Errors
    ///
    /// Returns [`crate::MetadataWireDecodeError::Budget`] when the document
    /// exceeds a shared JSON limit, or `InvalidJson` for syntax and envelope
    /// failures.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice(
        input: &[u8],
    ) -> Result<Self, crate::MetadataWireDecodeError> {
        Self::decode_json_slice_with_limits(input, MetadataLimits::default())
    }

    /// Decodes a strict metadata JSON envelope after applying `limits`.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete untrusted JSON input.
    /// * `limits` - Shared JSON limits for this decoding session.
    ///
    /// # Returns
    ///
    /// The decoded metadata object.
    ///
    /// # Errors
    ///
    /// Returns a structured budget error before or during decoding, or
    /// `InvalidJson` for syntax, strict-envelope, strict-map, or scalar
    /// wire-value failures.
    #[cfg(feature = "json")]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        limits: MetadataLimits,
    ) -> Result<Self, crate::MetadataWireDecodeError> {
        limits
            .validate()
            .map_err(crate::MetadataWireDecodeError::InvalidJson)?;
        let mut session = JsonDecodeSession::owned(limits.json_decode());
        let wire = decode_slice_seed(
            MetadataWireV1Seed::new(StrictStringMapSeed::new(
                limits.max_metadata_entries(),
                limits.max_key_bytes(),
            )),
            input,
            &mut session,
        )
        .map_err(metadata_json_error)?;
        if wire.version != METADATA_WIRE_VERSION_V1 {
            return Err(crate::MetadataWireDecodeError::InvalidJson(
                <serde_json::Error as DeError>::custom(
                    "unsupported Metadata wire format version",
                ),
            ));
        }
        let metadata = Self(Self::from_wire(wire).map_err(|error| {
            crate::MetadataWireDecodeError::InvalidJson(
                <serde_json::Error as DeError>::custom(error),
            )
        })?);
        Ok(metadata)
    }

    /// Encodes this metadata object with the default JSON budget profile.
    ///
    /// # Returns
    ///
    /// Compact JSON bytes accepted by the strict metadata wire format.
    ///
    /// # Errors
    ///
    /// Returns [`crate::MetadataWireEncodeError`] when the JSON value or
    /// output exceeds a configured budget, serialization fails, or the
    /// destination writer rejects bytes.
    #[cfg(feature = "json")]
    pub fn to_json_vec(
        &self,
    ) -> Result<Vec<u8>, crate::MetadataWireEncodeError> {
        self.to_json_vec_with_limits(
            crate::metadata_limits::default_json_encode_limits(),
        )
    }

    /// Encodes this metadata object with caller-provided JSON budgets.
    ///
    /// # Parameters
    ///
    /// * `limits` - Output and JSON-value budgets for this operation.
    ///
    /// # Returns
    ///
    /// Compact JSON bytes accepted by the strict metadata wire format.
    ///
    /// # Errors
    ///
    /// Returns [`crate::MetadataWireEncodeError`] when the JSON value or
    /// output exceeds a configured budget, or serialization fails.
    #[cfg(feature = "json")]
    pub fn to_json_vec_with_limits(
        &self,
        limits: JsonEncodeLimits,
    ) -> Result<Vec<u8>, crate::MetadataWireEncodeError> {
        let mut session = JsonEncodeSession::owned(limits);
        encode_to_vec(self, &mut session).map_err(Into::into)
    }

    /// Encodes this metadata object to a writer with the default JSON budget.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the complete compact JSON document.
    ///
    /// # Errors
    ///
    /// Returns [`crate::MetadataWireEncodeError`] when encoding exceeds a
    /// budget, serialization fails, or `writer` rejects the output.
    #[cfg(feature = "json")]
    pub fn to_json_writer<W>(
        &self,
        writer: W,
    ) -> Result<(), crate::MetadataWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(
            writer,
            crate::metadata_limits::default_json_encode_limits(),
        )
    }

    /// Encodes this metadata object to a writer with caller-provided budgets.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the complete compact JSON document.
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
        let mut session = JsonEncodeSession::owned(limits);
        encode_to_writer(writer, self, &mut session).map_err(Into::into)
    }

    /// Returns `true` if there are no entries.
    ///
    /// # Returns
    ///
    /// `true` when this object contains no entries.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of key-value pairs.
    ///
    /// # Returns
    ///
    /// The number of stored entries.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the given key exists, including when it stores
    /// [`Value::Unset`].
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to inspect.
    ///
    /// # Returns
    ///
    /// `true` when an entry exists for `key`.
    #[inline(always)]
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Retrieves the value associated with `key` and converts it to `T`.
    ///
    /// This convenience method returns `None` when the key is absent or when
    /// the stored [`Value`] cannot be converted to `T`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to retrieve.
    ///
    /// # Returns
    ///
    /// The converted value, or `None` when lookup or conversion fails.
    #[inline(always)]
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: DataConversionTarget,
    {
        self.try_get(key).ok()
    }

    /// Retrieves a borrowed string stored under `key`.
    ///
    /// This accessor only succeeds when the stored value is a concrete string;
    /// it does not allocate or perform a conversion.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to retrieve.
    ///
    /// # Returns
    ///
    /// The borrowed string, or `None` when the key is absent, unset, or stores
    /// another value type.
    #[inline(always)]
    #[must_use]
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.try_get_str(key).ok()
    }

    /// Retrieves a borrowed concrete string stored under `key`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to retrieve.
    ///
    /// # Returns
    ///
    /// The borrowed string stored under `key`.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::MissingKey`] when the key is absent,
    /// [`MetadataError::MissingValue`] when it stores [`Value::Unset`], or
    /// [`MetadataError::TypeMismatch`] when the stored value is not a string.
    #[inline(always)]
    pub fn try_get_str(&self, key: &str) -> MetadataResult<&str> {
        let value = self
            .0
            .get(key)
            .ok_or_else(|| MetadataError::MissingKey(key.to_string()))?;
        match value.view() {
            ValueRef::String(string) => Ok(string),
            ValueRef::Unset(data_type) => Err(MetadataError::MissingValue {
                key: key.to_string(),
                data_type,
            }),
            _ => Err(MetadataError::TypeMismatch {
                key: key.to_string(),
                expected: DataType::String,
                actual: value.data_type(),
                message: "stored value is not a string".to_string(),
            }),
        }
    }

    /// Retrieves the value associated with `key` and converts it to `T`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to retrieve.
    ///
    /// # Returns
    ///
    /// The stored value converted to `T`.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::MissingKey`] when the key is absent,
    /// [`MetadataError::MissingValue`] when it stores [`Value::Unset`], or
    /// [`MetadataError::TypeMismatch`] when the stored value cannot be
    /// converted to the requested type.
    pub fn try_get<T>(&self, key: &str) -> MetadataResult<T>
    where
        T: DataConversionTarget,
    {
        let value = self
            .0
            .get(key)
            .ok_or_else(|| MetadataError::MissingKey(key.to_string()))?;
        if value.is_unset() {
            return Err(MetadataError::MissingValue {
                key: key.to_string(),
                data_type: value.data_type(),
            });
        }
        value.to::<T>().map_err(|error| {
            MetadataError::conversion_error(key, T::DATA_TYPE, value, error)
        })
    }

    /// Returns a reference to the stored [`Value`] for `key`, or `None` if
    /// absent.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to retrieve.
    ///
    /// # Returns
    ///
    /// The stored value, or `None` when `key` is absent.
    #[inline(always)]
    pub fn get_raw(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    /// Returns the concrete data type of the value stored under `key`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to inspect.
    ///
    /// # Returns
    ///
    /// The stored value's data type, or `None` when `key` is absent.
    #[inline(always)]
    pub fn data_type(&self, key: &str) -> Option<DataType> {
        self.0.get(key).map(Value::data_type)
    }

    /// Retrieves and converts the value associated with `key`, or returns
    /// `default` if lookup or conversion fails.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to retrieve.
    /// * `default` - Value returned when lookup or conversion fails.
    ///
    /// # Returns
    ///
    /// The converted stored value or `default`.
    #[inline(always)]
    #[must_use]
    pub fn get_or<T>(&self, key: &str, default: T) -> T
    where
        T: DataConversionTarget,
    {
        self.try_get(key).unwrap_or(default)
    }

    /// Inserts a typed value and returns the previous value.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to replace.
    /// * `value` - Typed value to store.
    ///
    /// # Returns
    ///
    /// The previous value when the key was already present, or `None`.
    #[inline(always)]
    pub fn insert<T>(&mut self, key: &str, value: T) -> Option<Value>
    where
        T: Into<Value>,
    {
        self.0.insert(key.to_string(), value.into())
    }

    /// Sets a typed value and returns this metadata object for chaining.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to replace.
    /// * `value` - Typed value to store.
    ///
    /// # Returns
    ///
    /// A mutable reference to this metadata object.
    #[inline(always)]
    pub fn set<T>(&mut self, key: &str, value: T) -> &mut Self
    where
        T: Into<Value>,
    {
        let _ = self.insert(key, value);
        self
    }

    /// Returns a new metadata object with `key` set to `value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to replace.
    /// * `value` - Typed value to store.
    ///
    /// # Returns
    ///
    /// This metadata object after inserting the value.
    #[inline(always)]
    #[must_use]
    pub fn with<T>(mut self, key: &str, value: T) -> Self
    where
        T: Into<Value>,
    {
        self.set(key, value);
        self
    }

    /// Inserts a typed value after validating it against `schema` and returns
    /// the previous value.
    ///
    /// # Parameters
    ///
    /// * `schema` - Schema used to validate the entry.
    /// * `key` - Metadata key to replace.
    /// * `value` - Typed value to validate and store.
    ///
    /// # Returns
    ///
    /// The previous value when the key was already present, or `None`.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::UnknownField`] when `key` is rejected by the
    /// schema, [`MetadataError::MissingRequiredField`] when a required field is
    /// assigned [`Value::Unset`], or [`MetadataError::TypeMismatch`] when the
    /// constructed value's concrete type does not match the schema field type.
    #[cfg(feature = "schema")]
    #[inline]
    pub fn insert_checked<T>(
        &mut self,
        schema: &MetadataSchema,
        key: &str,
        value: T,
    ) -> MetadataResult<Option<Value>>
    where
        T: Into<Value>,
    {
        let value = value.into();
        schema.validate_entry(key, &value)?;
        Ok(self.insert(key, value))
    }

    /// Sets a typed value after schema validation and returns this metadata
    /// object for chaining.
    ///
    /// # Parameters
    ///
    /// * `schema` - Schema used to validate the entry.
    /// * `key` - Metadata key to replace.
    /// * `value` - Typed value to validate and store.
    ///
    /// # Returns
    ///
    /// A mutable reference to this metadata object.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::UnknownField`] when `key` is rejected by the
    /// schema, [`MetadataError::MissingRequiredField`] when a required field is
    /// assigned [`Value::Unset`], or [`MetadataError::TypeMismatch`] when the
    /// constructed value's concrete type does not match the schema field type.
    #[cfg(feature = "schema")]
    #[inline(always)]
    pub fn set_checked<T>(
        &mut self,
        schema: &MetadataSchema,
        key: &str,
        value: T,
    ) -> MetadataResult<&mut Self>
    where
        T: Into<Value>,
    {
        let _ = self.insert_checked(schema, key, value)?;
        Ok(self)
    }

    /// Returns a new metadata object with a typed value validated and inserted.
    ///
    /// # Parameters
    ///
    /// * `schema` - Schema used to validate the entry.
    /// * `key` - Metadata key to replace.
    /// * `value` - Typed value to validate and store.
    ///
    /// # Returns
    ///
    /// This metadata object after inserting the validated value.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::UnknownField`] when `key` is rejected by the
    /// schema, [`MetadataError::MissingRequiredField`] when a required field is
    /// assigned [`Value::Unset`], or [`MetadataError::TypeMismatch`] when the
    /// constructed value's concrete type does not match the schema field type.
    #[cfg(feature = "schema")]
    #[inline(always)]
    pub fn with_checked<T>(
        mut self,
        schema: &MetadataSchema,
        key: &str,
        value: T,
    ) -> MetadataResult<Self>
    where
        T: Into<Value>,
    {
        self.set_checked(schema, key, value)?;
        Ok(self)
    }

    /// Removes the entry for `key` and returns the stored [`Value`] if it
    /// existed.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to remove.
    ///
    /// # Returns
    ///
    /// The removed value, or `None` when `key` was absent.
    #[inline(always)]
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.remove(key)
    }

    /// Removes all entries.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns an iterator over `(&str, &Value)` pairs in key-sorted order.
    ///
    /// # Returns
    ///
    /// A borrowing iterator over entries in key order.
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.0.iter().map(|(key, value)| (key.as_str(), value))
    }

    /// Returns an iterator over the keys in sorted order.
    ///
    /// # Returns
    ///
    /// A borrowing iterator over keys in sorted order.
    #[inline(always)]
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }

    /// Returns an iterator over the values in key-sorted order.
    ///
    /// # Returns
    ///
    /// A borrowing iterator over values in key order.
    #[inline(always)]
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.0.values()
    }

    /// Merges all entries from `other` into `self`, overwriting existing keys.
    ///
    /// # Parameters
    ///
    /// * `other` - Metadata entries to consume and merge.
    pub fn merge(&mut self, mut other: Metadata) {
        self.0.append(&mut other.0);
    }

    /// Returns a new `Metadata` that contains entries from `self` and `other`.
    ///
    /// Entries from `other` take precedence on key conflicts.
    ///
    /// # Parameters
    ///
    /// * `other` - Metadata entries to merge.
    ///
    /// # Returns
    ///
    /// A merged copy without modifying either input.
    #[must_use]
    pub fn merged(&self, other: &Metadata) -> Metadata {
        let mut result = self.clone();
        let mut right = other.0.clone();
        result.0.append(&mut right);
        result
    }

    /// Retains only the entries for which `predicate` returns `true`.
    ///
    /// # Parameters
    ///
    /// * `predicate` - Callback invoked for each key and value; returning
    ///   `false` removes that entry.
    #[inline(always)]
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&str, &Value) -> bool,
    {
        self.0.retain(|key, value| predicate(key.as_str(), value));
    }

    /// Converts this metadata object into its underlying map.
    ///
    /// # Returns
    ///
    /// The owned, key-sorted map of metadata values.
    #[inline(always)]
    #[must_use]
    pub fn into_inner(self) -> BTreeMap<String, Value> {
        self.0
    }

    /// Validates that this metadata object fits the strict V1 wire contract.
    ///
    /// This preflight checks the same entry-count and key-byte limits enforced
    /// by [`Serialize::serialize`], allowing callers to reject invalid metadata
    /// at the write boundary instead of discovering the error during encoding.
    ///
    /// # Returns
    ///
    /// `Ok(())` when every entry can satisfy the metadata map limits.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::WireLimitExceeded`] when the entry count or a
    /// key exceeds the strict V1 limit.
    #[inline]
    pub fn validate_wire_contract(&self) -> MetadataResult<()> {
        if self.0.len() > STRICT_STRING_MAP_MAX_ENTRIES {
            return Err(MetadataError::WireLimitExceeded {
                kind: crate::MetadataWireLimitKind::Entries,
                value: self.0.len(),
                maximum: STRICT_STRING_MAP_MAX_ENTRIES,
            });
        }
        if let Some(key) = self
            .0
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

/// Converts a shared JSON adapter error into the metadata decoding error.
#[cfg(feature = "json")]
fn metadata_json_error(
    error: JsonDecodeError<JsonResource>,
) -> crate::MetadataWireDecodeError {
    match error {
        JsonDecodeError::Budget(error) => match error {
            MeasuredBudgetError::Budget(error) => {
                crate::MetadataWireDecodeError::Budget(error)
            }
            MeasuredBudgetError::Quantity { resource, source } => {
                crate::MetadataWireDecodeError::Quantity { resource, source }
            }
        },
        JsonDecodeError::Syntax(error) => {
            crate::MetadataWireDecodeError::Syntax(error)
        }
        JsonDecodeError::Deserialize(error) => {
            crate::MetadataWireDecodeError::InvalidJson(
                <serde_json::Error as DeError>::custom(error),
            )
        }
    }
}

impl Redact for Metadata {
    fn redaction_input_bytes(&self) -> usize {
        self.0.iter().fold(0, |total, (key, value)| {
            total
                .saturating_add(key.len())
                .saturating_add(Redact::redaction_input_bytes(value))
        })
    }

    /// Writes a policy-redacted metadata representation.
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let mut output = formatter.debug_map();
        for (key, value) in &self.0 {
            output.entry(key, &RedactedKeyedResult::new(key, value, session));
        }
        output.finish()
    }
}

impl fmt::Debug for Metadata {
    /// Writes the default-policy redacted representation.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.redacted().with_policy_output_limit(), formatter)
    }
}

impl fmt::Display for Metadata {
    /// Writes a bounded, default-policy redacted representation as
    /// single-line diagnostic text.
    ///
    /// The default policy is not a confidentiality boundary for arbitrary
    /// user-defined keys or error text; callers should apply an explicit
    /// redaction policy before emitting untrusted metadata to logs.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(
            &self.redacted().with_policy_output_limit(),
            formatter,
        )
    }
}

impl Serialize for Metadata {
    /// Serializes metadata as the strict v1 envelope.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.validate_wire_contract()
            .map_err(<S::Error as SerError>::custom)?;
        MetadataWireV1 {
            version: METADATA_WIRE_VERSION_V1,
            values: MetadataWireValuesRef(&self.0),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Metadata {
    /// Deserializes only the strict v1 envelope.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire: MetadataWireV1<StrictStringMap<ValueWirePayloadV1>> =
            MetadataWireV1::deserialize(deserializer)?;
        if wire.version != METADATA_WIRE_VERSION_V1 {
            return Err(de::Error::custom(
                "unsupported Metadata wire format version",
            ));
        }
        let values = Self::from_wire(wire).map_err(de::Error::custom)?;
        Ok(Self(values))
    }
}

impl Metadata {
    /// Converts a decoded wire envelope into scalar metadata values.
    fn from_wire(
        wire: MetadataWireV1<StrictStringMap<ValueWirePayloadV1>>,
    ) -> Result<BTreeMap<String, Value>, &'static str> {
        wire.values
            .into_inner()
            .into_iter()
            .map(|(key, value)| {
                value
                    .into_container()
                    .into_scalar()
                    .map(|value| (key, value))
                    .map_err(|_| "metadata values must use scalar V1 payloads")
            })
            .collect()
    }
}

impl From<BTreeMap<String, Value>> for Metadata {
    #[inline(always)]
    fn from(map: BTreeMap<String, Value>) -> Self {
        Self(map)
    }
}

impl From<Metadata> for BTreeMap<String, Value> {
    #[inline(always)]
    fn from(meta: Metadata) -> Self {
        meta.0
    }
}

impl FromIterator<(String, Value)> for Metadata {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for Metadata {
    type IntoIter = std::collections::btree_map::IntoIter<String, Value>;
    type Item = (String, Value);

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Metadata {
    type IntoIter = std::collections::btree_map::Iter<'a, String, Value>;
    type Item = (&'a String, &'a Value);

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Extend<(String, Value)> for Metadata {
    #[inline(always)]
    fn extend<I: IntoIterator<Item = (String, Value)>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}
