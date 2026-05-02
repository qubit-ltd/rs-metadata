/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Provides the [`Metadata`] type — a structured, ordered, typed key-value store.

use std::collections::BTreeMap;

use qubit_datatype::{DataType, DataTypeOf};
use qubit_value::{Value, ValueConstructor, ValueConverter};
use serde::{Deserialize, Serialize};

use crate::{MetadataError, MetadataResult, MetadataSchema};

/// A structured, ordered, typed key-value store for metadata fields.
///
/// `Metadata` stores values as [`qubit_value::Value`], preserving concrete Rust
/// scalar types such as `i64`, `u32`, `f64`, `String`, and `bool`.  This avoids
/// the ambiguity of a single JSON number type while still allowing callers to
/// store explicit [`Value::Json`] values when they really need JSON payloads.
///
/// Use [`Metadata::with`] for fluent construction and [`Metadata::set`] when
/// mutating an existing object.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Metadata(BTreeMap<String, Value>);

impl Metadata {
    /// Creates an empty metadata object.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Returns `true` if there are no entries.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of key-value pairs.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the given key exists.
    #[inline]
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Retrieves the value associated with `key` and converts it to `T`.
    ///
    /// This convenience method returns `None` when the key is absent or when the
    /// stored [`Value`] cannot be converted to `T`.
    #[inline]
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: DataTypeOf,
        Value: ValueConverter<T>,
    {
        self.try_get(key).ok()
    }

    /// Retrieves the value associated with `key` and converts it to `T`.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::MissingKey`] when the key is absent, or
    /// [`MetadataError::TypeMismatch`] when the stored value cannot be converted
    /// to the requested type.
    pub fn try_get<T>(&self, key: &str) -> MetadataResult<T>
    where
        T: DataTypeOf,
        Value: ValueConverter<T>,
    {
        let value = self
            .0
            .get(key)
            .ok_or_else(|| MetadataError::MissingKey(key.to_string()))?;
        value
            .to::<T>()
            .map_err(|error| MetadataError::conversion_error(key, T::DATA_TYPE, value, error))
    }

    /// Returns a reference to the stored [`Value`] for `key`, or `None` if absent.
    #[inline]
    #[must_use]
    pub fn get_raw(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    /// Returns the concrete data type of the value stored under `key`.
    #[inline]
    #[must_use]
    pub fn data_type(&self, key: &str) -> Option<DataType> {
        self.0.get(key).map(Value::data_type)
    }

    /// Retrieves and converts the value associated with `key`, or returns
    /// `default` if lookup or conversion fails.
    #[inline]
    #[must_use]
    pub fn get_or<T>(&self, key: &str, default: T) -> T
    where
        T: DataTypeOf,
        Value: ValueConverter<T>,
    {
        self.try_get(key).unwrap_or(default)
    }

    /// Inserts a typed value under `key` and returns the previous value if present.
    #[inline]
    pub fn set<T>(&mut self, key: &str, value: T) -> Option<Value>
    where
        Value: ValueConstructor<T>,
    {
        self.0.insert(key.to_string(), to_value(value))
    }

    /// Inserts a typed value after validating it against `schema`.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::UnknownField`] when `key` is rejected by the
    /// schema, or [`MetadataError::TypeMismatch`] when the constructed value's
    /// concrete type does not match the schema field type.
    #[inline]
    pub fn set_checked<T>(
        &mut self,
        schema: &MetadataSchema,
        key: &str,
        value: T,
    ) -> MetadataResult<Option<Value>>
    where
        Value: ValueConstructor<T>,
    {
        let value = to_value(value);
        schema.validate_entry(key, &value)?;
        Ok(self.set_raw(key, value))
    }

    /// Returns a new metadata object with a typed value validated and inserted.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::UnknownField`] when `key` is rejected by the
    /// schema, or [`MetadataError::TypeMismatch`] when the constructed value's
    /// concrete type does not match the schema field type.
    #[inline]
    pub fn with_checked<T>(
        mut self,
        schema: &MetadataSchema,
        key: &str,
        value: T,
    ) -> MetadataResult<Self>
    where
        Value: ValueConstructor<T>,
    {
        self.set_checked(schema, key, value)?;
        Ok(self)
    }

    /// Returns a new metadata object with `key` set to `value`.
    #[inline]
    #[must_use]
    pub fn with<T>(mut self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.set(key, value);
        self
    }

    /// Inserts a raw [`Value`] directly and returns the previous value if present.
    #[inline]
    pub fn set_raw(&mut self, key: &str, value: Value) -> Option<Value> {
        self.0.insert(key.to_string(), value)
    }

    /// Returns a new metadata object with a raw [`Value`] inserted.
    #[inline]
    #[must_use]
    pub fn with_raw(mut self, key: &str, value: Value) -> Self {
        self.set_raw(key, value);
        self
    }

    /// Removes the entry for `key` and returns the stored [`Value`] if it existed.
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
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.0.iter().map(|(key, value)| (key.as_str(), value))
    }

    /// Returns an iterator over the keys in sorted order.
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }

    /// Returns an iterator over the values in key-sorted order.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.0.values()
    }

    /// Merges all entries from `other` into `self`, overwriting existing keys.
    pub fn merge(&mut self, other: Metadata) {
        for (key, value) in other.0 {
            self.0.insert(key, value);
        }
    }

    /// Returns a new `Metadata` that contains entries from `self` and `other`.
    ///
    /// Entries from `other` take precedence on key conflicts.
    #[must_use]
    pub fn merged(&self, other: &Metadata) -> Metadata {
        let mut result = self.clone();
        for (key, value) in &other.0 {
            result.0.insert(key.clone(), value.clone());
        }
        result
    }

    /// Retains only the entries for which `predicate` returns `true`.
    #[inline]
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&str, &Value) -> bool,
    {
        self.0.retain(|key, value| predicate(key.as_str(), value));
    }

    /// Converts this metadata object into its underlying map.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> BTreeMap<String, Value> {
        self.0
    }
}

#[inline]
fn to_value<T>(value: T) -> Value
where
    Value: ValueConstructor<T>,
{
    <Value as ValueConstructor<T>>::from_type(value)
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
