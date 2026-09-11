// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provides the [`Metadata`] type — a structured, key-sorted, typed key-value
//! store.

#[cfg(feature = "json")]
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
#[cfg(feature = "json")]
use std::io::Write;
#[cfg(feature = "json")]
use std::rc::Rc;

#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeSession;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeSession;
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::DataConversionTarget;
use qubit_datatype::DataType;
#[cfg(feature = "json")]
use qubit_json::decode::JsonDecoder;
#[cfg(feature = "json")]
use qubit_json::encode::JsonEncoder;
use qubit_redact::Redact;
use qubit_redact::RedactionWriter;
use qubit_redact::Redactor;
use qubit_value::IntoValueDefault;
use qubit_value::StrictValueRead;
use qubit_value::Value;
use qubit_value::ValueError;
#[cfg(feature = "json")]
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWirePayloadV1Seed;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de;
use serde::de::DeserializeSeed;
#[cfg(feature = "json")]
use serde::de::Error as DeError;
use serde::ser::Error as SerError;

use crate::MetadataError;
use crate::MetadataResult;
#[cfg(feature = "schema")]
use crate::MetadataSchema;
use crate::constants::STRICT_STRING_MAP_MAX_ENTRIES;
use crate::constants::STRICT_STRING_MAP_MAX_KEY_BYTES;
use crate::internal::MetadataValues;
#[cfg(feature = "json")]
use crate::metadata_limits::MetadataLimits;
use crate::wire::METADATA_WIRE_VERSION_V1;
use crate::wire::MetadataWireV1;
use crate::wire::MetadataWireV1Seed;
use crate::wire::MetadataWireValuesRef;
use crate::wire::StrictStringMap;
use crate::wire::StrictStringMapValueSeed;

/// A structured, key-sorted, typed key-value store for metadata fields.
///
/// `Metadata` stores values as [`qubit_value::Value`], preserving concrete Rust
/// scalar types such as `i64`, `u32`, `f64`, `String`, and `bool`.  This avoids
/// the ambiguity of a single JSON number type while still allowing callers to
/// store explicit `Value::Json` values when they really need JSON payloads.
/// [`Value::Unset`] retains a declared type but represents no concrete metadata
/// value: typed reads report [`MetadataError::ValueAccess`]. When the optional
/// `schema` or `filter` features are enabled, their validation and matching
/// APIs treat it as a missing concrete value.
///
/// Use [`Metadata::with`] for fluent construction and [`Metadata::set`] when
/// mutating an existing object. The typed [`Metadata::get`] and
/// [`Metadata::get_ref`] accessors read strictly; [`Metadata::convert`]
/// explicitly converts stored values. Use [`Metadata::get_raw`] when
/// the stored runtime [`qubit_value::Value`] must be inspected without
/// conversion.
///
/// # Examples
///
/// ```
/// use qubit_metadata::Metadata;
///
/// # fn main() -> qubit_metadata::MetadataResult<()> {
/// let metadata = Metadata::new().with("tenant", "acme");
/// assert_eq!(metadata.get_ref::<str>("tenant")?, "acme");
/// # Ok(())
/// # }
/// ```
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
    /// failures. Domain limits return `Domain`; unsupported versions return
    /// `UnsupportedVersion`.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice(input: &[u8]) -> Result<Self, crate::MetadataWireDecodeError> {
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
    /// Returns a structured budget error before or during decoding, `Domain`
    /// for entry/key limits, `UnsupportedVersion` for a version mismatch, or
    /// redacted JSON errors for syntax, envelope, and scalar wire failures.
    #[cfg(feature = "json")]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        limits: MetadataLimits,
    ) -> Result<Self, crate::MetadataWireDecodeError> {
        limits
            .validate()
            .map_err(crate::MetadataWireDecodeError::InvalidLimits)?;
        let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(limits.json_decode()));
        let error_slot = Rc::new(RefCell::new(None));
        let wire = decoder
            .decode_seed_utf8(
                MetadataWireV1Seed::new(
                    StrictStringMapValueSeed::new(
                        limits.max_metadata_entries(),
                        limits.max_key_bytes(),
                        ValueWirePayloadV1Seed::new(),
                    )
                    .with_error_slot(Rc::clone(&error_slot)),
                ),
                input,
            )
            .map_err(|error| {
                error_slot.borrow_mut().take().map_or_else(
                    || Into::<crate::MetadataWireDecodeError>::into(error),
                    crate::MetadataWireDecodeError::Domain,
                )
            })?;
        if wire.version != METADATA_WIRE_VERSION_V1 {
            return Err(crate::MetadataWireDecodeError::UnsupportedVersion {
                expected: METADATA_WIRE_VERSION_V1,
                actual: wire.version,
            });
        }
        let metadata = Self(Self::from_wire(wire).map_err(|error| {
            crate::MetadataWireDecodeError::InvalidJson(<serde_json::Error as DeError>::custom(error))
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
    pub fn to_json_vec(&self) -> Result<Vec<u8>, crate::MetadataWireEncodeError> {
        self.to_json_vec_with_limits(crate::metadata_limits::default_json_encode_limits())
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
    pub fn to_json_vec_with_limits(&self, limits: JsonEncodeLimits) -> Result<Vec<u8>, crate::MetadataWireEncodeError> {
        let mut preflight = ValueWireEncodePreflight::new_value_limits(*limits.value_limits());
        for value in self.0.values() {
            preflight
                .check_value(value)
                .map_err(crate::MetadataWireEncodeError::from)?;
        }
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session).to_vec(self).map_err(Into::into)
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
    pub fn to_json_writer<W>(&self, writer: W) -> Result<(), crate::MetadataWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(writer, crate::metadata_limits::default_json_encode_limits())
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
        let mut preflight = ValueWireEncodePreflight::new_value_limits(*limits.value_limits());
        for value in self.0.values() {
            preflight
                .check_value(value)
                .map_err(crate::MetadataWireEncodeError::from)?;
        }
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .write_buffered(writer, self)
            .map_err(Into::into)
    }

    /// Returns `true` if there are no entries.
    ///
    /// # Returns
    ///
    /// `true` when this object contains no entries.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of key-value pairs.
    ///
    /// # Returns
    ///
    /// The number of stored entries.
    #[inline]
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
    #[inline]
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Strictly reads `key` as `T`, without coercing the stored runtime type.
    ///
    /// # Errors
    /// Returns MissingKey for an absent key, or ValueAccess preserving type
    /// mismatch and unset facts. Use [`Self::convert`] for coercing reads.
    pub fn get<T: StrictValueRead>(&self, key: &str) -> MetadataResult<T> {
        T::read_scalar(self.entry(key)?).map_err(|source| Self::map_access_error(key, source))
    }

    /// Borrows the concrete payload under `key` without copying it.
    ///
    /// # Errors
    /// Returns MissingKey or ValueAccess for unset storage or a type mismatch.
    /// The returned reference borrows this metadata object, not the key.
    pub fn get_ref<'a, T: ?Sized>(&'a self, key: &str) -> MetadataResult<&'a T>
    where
        &'a T: TryFrom<&'a Value, Error = ValueError>,
    {
        self.entry(key)?
            .get_ref::<T>()
            .map_err(|source| Self::map_access_error(key, source))
    }

    /// Strictly reads `key`, returning None only for absent or matching unset
    /// storage.
    ///
    /// # Errors
    /// Preserves type mismatches, including unset storage of a different type.
    pub fn get_optional<T: StrictValueRead>(&self, key: &str) -> MetadataResult<Option<T>> {
        match self.get(key) {
            Ok(value) => Ok(Some(value)),
            Err(error) if Self::is_defaultable(&error, false) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Strictly reads `key`, adapting `default` only for absent or matching
    /// unset storage.
    ///
    /// # Errors
    /// Returns the original read error for a type mismatch; never hides invalid
    /// data.
    pub fn get_or<T: StrictValueRead>(&self, key: &str, default: impl IntoValueDefault<T>) -> MetadataResult<T> {
        self.get_optional(key)
            .map(|value| value.unwrap_or_else(|| default.into_value_default()))
    }

    /// Converts `key` to `T` using the default conversion policy and limits.
    ///
    /// # Errors
    /// Returns MissingKey or ValueAccess with the original missing, invalid,
    /// unsupported, precision-loss or resource error and its source chain.
    pub fn convert<T: DataConversionTarget>(&self, key: &str) -> MetadataResult<T> {
        self.convert_with(key, ConversionPolicy::default_ref(), ConversionLimits::default_ref())
    }

    /// Converts `key` to `T` with explicit `policy` and `limits`.
    ///
    /// Each call owns one conversion budget and leaves stored data unchanged.
    ///
    /// # Errors
    /// Returns MissingKey or ValueAccess preserving conversion facts and
    /// limits.
    pub fn convert_with<T: DataConversionTarget>(
        &self,
        key: &str,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> MetadataResult<T> {
        self.entry(key)?
            .to_with(policy, limits)
            .map_err(|source| Self::map_access_error(key, source))
    }

    /// Converts `key`, returning None for absent, unset, or policy-missing
    /// scalars.
    ///
    /// # Errors
    /// All other conversion errors propagate with their source and key.
    pub fn convert_optional_with<T: DataConversionTarget>(
        &self,
        key: &str,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> MetadataResult<Option<T>> {
        match self.convert_with(key, policy, limits) {
            Ok(value) => Ok(Some(value)),
            Err(error) if Self::is_defaultable(&error, true) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Converts `key`, adapting `default` only for a defaultable missing
    /// scalar.
    ///
    /// # Errors
    /// Invalid, unsupported, precision-loss and resource errors never default.
    pub fn convert_or_with<T: DataConversionTarget>(
        &self,
        key: &str,
        default: impl IntoValueDefault<T>,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> MetadataResult<T> {
        self.convert_optional_with(key, policy, limits)
            .map(|value| value.unwrap_or_else(|| default.into_value_default()))
    }

    /// Looks up storage, leaving unset and type classification to the value
    /// layer.
    fn entry(&self, key: &str) -> MetadataResult<&Value> {
        self.get_raw(key)
            .ok_or_else(|| MetadataError::MissingKey(key.to_owned()))
    }

    /// Attaches a key without flattening or replacing the original value error.
    fn map_access_error(key: &str, source: ValueError) -> MetadataError {
        MetadataError::ValueAccess {
            key: key.to_owned(),
            source: Box::new(source),
        }
    }

    /// Applies the value layer's strict or conversion fallback classification.
    fn is_defaultable(error: &MetadataError, conversion: bool) -> bool {
        match error {
            MetadataError::MissingKey(_) => true,
            MetadataError::ValueAccess { source, .. } => source.missing().is_some_and(|missing| {
                if conversion {
                    missing.is_defaultable_for_conversion()
                } else {
                    missing.is_defaultable_for_strict_read()
                }
            }),
            _ => false,
        }
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
    #[inline]
    #[must_use]
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
    #[inline]
    #[must_use]
    pub fn data_type(&self, key: &str) -> Option<DataType> {
        self.0.get(key).map(Value::data_type)
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    pub fn insert_checked<T>(&mut self, schema: &MetadataSchema, key: &str, value: T) -> MetadataResult<Option<Value>>
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
    #[inline]
    pub fn set_checked<T>(&mut self, schema: &MetadataSchema, key: &str, value: T) -> MetadataResult<&mut Self>
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
    #[inline]
    pub fn with_checked<T>(mut self, schema: &MetadataSchema, key: &str, value: T) -> MetadataResult<Self>
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
    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.remove(key)
    }

    /// Removes all entries.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns an iterator over `(&str, &Value)` pairs in key-sorted order.
    ///
    /// # Returns
    ///
    /// A borrowing iterator over entries in key order.
    #[inline]
    #[must_use = "the metadata iterator must be consumed to inspect entries"]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.0.iter().map(|(key, value)| (key.as_str(), value))
    }

    /// Returns an iterator over the keys in sorted order.
    ///
    /// # Returns
    ///
    /// A borrowing iterator over keys in sorted order.
    #[inline]
    #[must_use = "the metadata key iterator must be consumed to inspect keys"]
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }

    /// Returns an iterator over the values in key-sorted order.
    ///
    /// # Returns
    ///
    /// A borrowing iterator over values in key order.
    #[inline]
    #[must_use = "the metadata value iterator must be consumed to inspect values"]
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
    #[inline]
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
    #[inline]
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
        if let Some(key) = self.0.keys().find(|key| key.len() > STRICT_STRING_MAP_MAX_KEY_BYTES) {
            return Err(MetadataError::WireLimitExceeded {
                kind: crate::MetadataWireLimitKind::KeyBytes,
                value: key.len(),
                maximum: STRICT_STRING_MAP_MAX_KEY_BYTES,
            });
        }
        Ok(())
    }
}

impl Redact for Metadata {
    /// Writes a policy-redacted metadata representation.
    ///
    /// Metadata is pure domain structure, so this traversal consumes nodes,
    /// collection items, and output bytes but no diagnostic input bytes. The
    /// metadata node and its map field are admitted before the stored map is
    /// accessed. The map writer then enters exactly one map node and admits
    /// each exact remaining entry before iterator advancement. Its admitted-
    /// item path classifies entries without charging duplicate keyed root and
    /// field nodes; pass-through values still enter their legitimate nested
    /// value scopes.
    ///
    /// # Parameters
    ///
    /// * `session` - Shared policy and cumulative diagnostic budgets.
    /// * `formatter` - Destination formatting context.
    ///
    /// # Returns
    ///
    /// The formatter result for the admitted safe map representation.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] when the destination rejects safe output.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        writer.record("Metadata", |fields| {
            fields.nested("values", &MetadataValues(&self.0));
        });
    }
}

impl fmt::Debug for Metadata {
    /// Writes the strict-policy redacted representation.
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = Redactor::strict().redact_text(self);
        let text = output.text_or_marker("<redaction incomplete>");
        formatter.write_str(text.as_ref())
    }
}

impl fmt::Display for Metadata {
    /// Writes a bounded, strict-policy redacted representation as
    /// single-line diagnostic text.
    ///
    /// The strict policy protects arbitrary user-defined keys and error text
    /// at this diagnostic boundary.
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = Redactor::strict().redact_text(self);
        let text = output.text_or_marker("<redaction incomplete>");
        formatter.write_str(text.as_ref())
    }
}

impl Serialize for Metadata {
    /// Serializes metadata as the strict v1 envelope.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.validate_wire_contract().map_err(<S::Error as SerError>::custom)?;
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
            MetadataWireV1Seed::new(StrictStringMapValueSeed::new(
                STRICT_STRING_MAP_MAX_ENTRIES,
                STRICT_STRING_MAP_MAX_KEY_BYTES,
                ValueWirePayloadV1Seed::new(),
            ))
            .deserialize(deserializer)?;
        if wire.version != METADATA_WIRE_VERSION_V1 {
            return Err(de::Error::custom("unsupported Metadata wire format version"));
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
    #[inline]
    fn from(map: BTreeMap<String, Value>) -> Self {
        Self(map)
    }
}

impl From<Metadata> for BTreeMap<String, Value> {
    #[inline]
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

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Metadata {
    type IntoIter = std::collections::btree_map::Iter<'a, String, Value>;
    type Item = (&'a String, &'a Value);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Extend<(String, Value)> for Metadata {
    #[inline]
    fn extend<I: IntoIterator<Item = (String, Value)>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}
