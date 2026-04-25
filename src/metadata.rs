/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Provides the [`Metadata`] type — a structured, ordered, typed key-value store.

use std::collections::BTreeMap;

use qubit_common::{DataType, DataTypeOf};
use qubit_value::{Value, ValueConstructor, ValueConverter};
use serde::{Deserialize, Serialize};

use crate::{Condition, MetadataError, MetadataFilter, MetadataResult};

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

    /// Inserts a typed value under `key`, preserving the `try_*` API shape.
    ///
    /// # Errors
    ///
    /// This method currently cannot fail for supported [`Value`] constructor
    /// types. It returns `Result` for symmetry with [`Metadata::try_get`] and to
    /// keep validation-oriented call sites explicit.
    #[inline]
    pub fn try_set<T>(&mut self, key: &str, value: T) -> MetadataResult<Option<Value>>
    where
        Value: ValueConstructor<T>,
    {
        Ok(self.set(key, value))
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

/// Policy for fields that appear in metadata but are not declared by a schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UnknownFieldPolicy {
    /// Reject fields that are not declared in the schema.
    #[default]
    Reject,
    /// Allow fields that are not declared in the schema.
    Allow,
}

/// Definition of one metadata field in a [`MetadataSchema`].
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

/// Schema for metadata fields.
///
/// A schema declares valid keys, their concrete [`DataType`], and whether they
/// are required. It can validate actual [`Metadata`] values and validate that a
/// [`MetadataFilter`] references known fields with compatible operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataSchema {
    /// Field definitions keyed by metadata key.
    fields: BTreeMap<String, MetadataField>,
    /// How validation handles unknown metadata keys.
    unknown_field_policy: UnknownFieldPolicy,
}

impl MetadataSchema {
    /// Creates a schema builder.
    #[inline]
    #[must_use]
    pub fn builder() -> MetadataSchemaBuilder {
        MetadataSchemaBuilder::default()
    }

    /// Returns the field definition for `key`.
    #[inline]
    #[must_use]
    pub fn field(&self, key: &str) -> Option<&MetadataField> {
        self.fields.get(key)
    }

    /// Returns the declared data type for `key`.
    #[inline]
    #[must_use]
    pub fn field_type(&self, key: &str) -> Option<DataType> {
        self.field(key).map(MetadataField::data_type)
    }

    /// Returns the unknown-field policy.
    #[inline]
    #[must_use]
    pub fn unknown_field_policy(&self) -> UnknownFieldPolicy {
        self.unknown_field_policy
    }

    /// Returns an iterator over schema fields in key-sorted order.
    #[inline]
    pub fn fields(&self) -> impl Iterator<Item = (&str, &MetadataField)> {
        self.fields.iter().map(|(key, field)| (key.as_str(), field))
    }

    /// Validates a metadata object against this schema.
    ///
    /// # Errors
    ///
    /// Returns an error when a required field is missing, a declared field has a
    /// different concrete type, or an unknown field is present while the schema
    /// rejects unknown fields.
    pub fn validate(&self, meta: &Metadata) -> MetadataResult<()> {
        for (key, field) in &self.fields {
            if field.required && !meta.contains_key(key) {
                return Err(MetadataError::MissingRequiredField {
                    key: key.clone(),
                    expected: field.data_type,
                });
            }
        }

        for (key, value) in meta.iter() {
            match self.field(key) {
                Some(field) if field.data_type != value.data_type() => {
                    return Err(MetadataError::type_mismatch(
                        key,
                        field.data_type,
                        value.data_type(),
                    ));
                }
                Some(_) => {}
                None if matches!(self.unknown_field_policy, UnknownFieldPolicy::Reject) => {
                    return Err(MetadataError::UnknownField {
                        key: key.to_string(),
                    });
                }
                None => {}
            }
        }
        Ok(())
    }

    /// Validates a metadata filter against this schema.
    ///
    /// # Errors
    ///
    /// Returns an error when the filter references an unknown field, uses a range
    /// operator on a non-comparable field, or compares a field with an incompatible
    /// value type.
    pub fn validate_filter(&self, filter: &MetadataFilter) -> MetadataResult<()> {
        filter.visit_conditions(|condition| self.validate_condition(condition))
    }

    /// Validates one filter condition against this schema.
    fn validate_condition(&self, condition: &Condition) -> MetadataResult<()> {
        match condition {
            Condition::Equal { key, value } | Condition::NotEqual { key, value } => {
                self.validate_value_condition(key, "eq", value)
            }
            Condition::Less { key, value } => self.validate_range_condition(key, "lt", value),
            Condition::LessEqual { key, value } => self.validate_range_condition(key, "le", value),
            Condition::Greater { key, value } => self.validate_range_condition(key, "gt", value),
            Condition::GreaterEqual { key, value } => {
                self.validate_range_condition(key, "ge", value)
            }
            Condition::In { key, values } | Condition::NotIn { key, values } => {
                for value in values {
                    self.validate_value_condition(key, "in_set", value)?;
                }
                Ok(())
            }
            Condition::Exists { key } | Condition::NotExists { key } => {
                self.require_field(key)?;
                Ok(())
            }
        }
    }

    /// Validates a non-range value condition.
    fn validate_value_condition(
        &self,
        key: &str,
        operator: &'static str,
        value: &Value,
    ) -> MetadataResult<()> {
        let field = self.require_field(key)?;
        if value_matches_field_type(value, field.data_type) {
            return Ok(());
        }
        Err(MetadataError::InvalidFilterOperator {
            key: key.to_string(),
            operator,
            data_type: field.data_type,
            message: format!(
                "filter value type {} is not compatible with field type {}",
                value.data_type(),
                field.data_type
            ),
        })
    }

    /// Validates a range value condition.
    fn validate_range_condition(
        &self,
        key: &str,
        operator: &'static str,
        value: &Value,
    ) -> MetadataResult<()> {
        let field = self.require_field(key)?;
        if !is_range_comparable_type(field.data_type) {
            return Err(MetadataError::InvalidFilterOperator {
                key: key.to_string(),
                operator,
                data_type: field.data_type,
                message: "range operators require a numeric or string field".to_string(),
            });
        }
        if value_matches_field_type(value, field.data_type) {
            return Ok(());
        }
        Err(MetadataError::InvalidFilterOperator {
            key: key.to_string(),
            operator,
            data_type: field.data_type,
            message: format!(
                "filter value type {} is not compatible with field type {}",
                value.data_type(),
                field.data_type
            ),
        })
    }

    /// Returns the field for `key` or a schema error if it is unknown.
    fn require_field(&self, key: &str) -> MetadataResult<&MetadataField> {
        self.field(key)
            .ok_or_else(|| MetadataError::UnknownFilterField {
                key: key.to_string(),
            })
    }
}

impl Default for MetadataSchema {
    #[inline]
    fn default() -> Self {
        Self {
            fields: BTreeMap::new(),
            unknown_field_policy: UnknownFieldPolicy::Reject,
        }
    }
}

/// Builder for [`MetadataSchema`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataSchemaBuilder {
    /// Field definitions being built.
    fields: BTreeMap<String, MetadataField>,
    /// Unknown-field policy copied into the built schema.
    unknown_field_policy: UnknownFieldPolicy,
}

impl MetadataSchemaBuilder {
    /// Adds a required field definition.
    #[inline]
    #[must_use]
    pub fn required(mut self, key: &str, data_type: DataType) -> Self {
        self.fields
            .insert(key.to_string(), MetadataField::new(data_type, true));
        self
    }

    /// Adds an optional field definition.
    #[inline]
    #[must_use]
    pub fn optional(mut self, key: &str, data_type: DataType) -> Self {
        self.fields
            .insert(key.to_string(), MetadataField::new(data_type, false));
        self
    }

    /// Sets the policy for metadata keys not declared by the schema.
    #[inline]
    #[must_use]
    pub fn unknown_field_policy(mut self, policy: UnknownFieldPolicy) -> Self {
        self.unknown_field_policy = policy;
        self
    }

    /// Builds the schema.
    #[inline]
    #[must_use]
    pub fn build(self) -> MetadataSchema {
        MetadataSchema {
            fields: self.fields,
            unknown_field_policy: self.unknown_field_policy,
        }
    }
}

/// Returns `true` when `data_type` is numeric.
#[inline]
fn is_numeric_data_type(data_type: DataType) -> bool {
    matches!(
        data_type,
        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::Int128
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::UInt128
            | DataType::Float32
            | DataType::Float64
            | DataType::BigInteger
            | DataType::BigDecimal
            | DataType::IntSize
            | DataType::UIntSize
    )
}

/// Returns `true` when `data_type` supports range comparisons.
#[inline]
fn is_range_comparable_type(data_type: DataType) -> bool {
    is_numeric_data_type(data_type) || matches!(data_type, DataType::String)
}

/// Returns `true` when a filter value is compatible with a schema field type.
#[inline]
fn value_matches_field_type(value: &Value, field_type: DataType) -> bool {
    let value_type = value.data_type();
    value_type == field_type || is_numeric_data_type(value_type) && is_numeric_data_type(field_type)
}
