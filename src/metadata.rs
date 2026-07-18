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

use qubit_datatype::{
    DataConversionTarget,
    DataType,
};
use qubit_value::Value;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    IntoMetadataValue,
    MetadataError,
    MetadataResult,
    MetadataSchema,
};

/// A structured, ordered, typed key-value store for metadata fields.
///
/// `Metadata` stores values as [`qubit_value::Value`], preserving concrete Rust
/// scalar types such as `i64`, `u32`, `f64`, `String`, and `bool`.  This avoids
/// the ambiguity of a single JSON number type while still allowing callers to
/// store explicit [`Value::Json`] values when they really need JSON payloads.
/// [`Value::Unset`] retains a declared type but represents no concrete metadata
/// value: typed reads report [`MetadataError::MissingValue`], required schema
/// fields reject it, and filters treat it like an absent key.
///
/// Use [`Metadata::with`] for fluent construction and [`Metadata::set`] when
/// mutating an existing object.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
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
        T: IntoMetadataValue,
    {
        self.0.insert(key.to_string(), value.into_metadata_value())
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
        T: IntoMetadataValue,
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
        T: IntoMetadataValue,
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
    /// schema, or [`MetadataError::TypeMismatch`] when the constructed value's
    /// concrete type does not match the schema field type.
    #[inline]
    pub fn insert_checked<T>(
        &mut self,
        schema: &MetadataSchema,
        key: &str,
        value: T,
    ) -> MetadataResult<Option<Value>>
    where
        T: IntoMetadataValue,
    {
        let value = value.into_metadata_value();
        schema.validate_entry(key, &value)?;
        Ok(self.insert_raw(key, value))
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
    /// schema, or [`MetadataError::TypeMismatch`] when the constructed value's
    /// concrete type does not match the schema field type.
    #[inline(always)]
    pub fn set_checked<T>(
        &mut self,
        schema: &MetadataSchema,
        key: &str,
        value: T,
    ) -> MetadataResult<&mut Self>
    where
        T: IntoMetadataValue,
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
    /// schema, or [`MetadataError::TypeMismatch`] when the constructed value's
    /// concrete type does not match the schema field type.
    #[inline(always)]
    pub fn with_checked<T>(
        mut self,
        schema: &MetadataSchema,
        key: &str,
        value: T,
    ) -> MetadataResult<Self>
    where
        T: IntoMetadataValue,
    {
        self.set_checked(schema, key, value)?;
        Ok(self)
    }

    /// Inserts a raw [`Value`] and returns the previous value.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to replace.
    /// * `value` - Raw value to store.
    ///
    /// # Returns
    ///
    /// The previous value when the key was already present, or `None`.
    #[inline(always)]
    pub fn insert_raw(&mut self, key: &str, value: Value) -> Option<Value> {
        self.0.insert(key.to_string(), value)
    }

    /// Sets a raw [`Value`] and returns this metadata object for chaining.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to replace.
    /// * `value` - Raw value to store.
    ///
    /// # Returns
    ///
    /// A mutable reference to this metadata object.
    #[inline(always)]
    pub fn set_raw(&mut self, key: &str, value: Value) -> &mut Self {
        let _ = self.insert_raw(key, value);
        self
    }

    /// Returns a new metadata object with a raw [`Value`] inserted.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to replace.
    /// * `value` - Raw value to store.
    ///
    /// # Returns
    ///
    /// This metadata object after inserting the value.
    #[inline(always)]
    #[must_use]
    pub fn with_raw(mut self, key: &str, value: Value) -> Self {
        self.set_raw(key, value);
        self
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
    pub fn merge(&mut self, other: Metadata) {
        for (key, value) in other.0 {
            let _ = self.0.insert(key, value);
        }
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
        for (key, value) in &other.0 {
            let _ = result.0.insert(key.clone(), value.clone());
        }
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
