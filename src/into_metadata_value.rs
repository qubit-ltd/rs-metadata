/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Provides conversion from supported Rust values into metadata values.

use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{
    DateTime,
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
    Utc,
};
use qubit_value::Value;
use url::Url;

/// Converts supported Rust values into the metadata backing [`Value`] type.
///
/// This trait is implemented for the same concrete input types accepted by
/// [`Value::new`]. It exists so [`crate::Metadata`] can keep a generic public
/// API while `qubit-value` keeps its internal constructor trait private.
pub trait IntoMetadataValue {
    /// Converts this value into a [`Value`] without changing its concrete data type.
    ///
    /// # Returns
    ///
    /// Returns the corresponding [`Value`] variant for this input type.
    fn into_metadata_value(self) -> Value;
}

macro_rules! impl_into_metadata_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl IntoMetadataValue for $type {
                #[inline]
                fn into_metadata_value(self) -> Value {
                    Value::new(self)
                }
            }
        )+
    };
}

impl_into_metadata_value!(
    bool,
    char,
    i8,
    i16,
    i32,
    i64,
    i128,
    u8,
    u16,
    u32,
    u64,
    u128,
    f32,
    f64,
    String,
    &str,
    NaiveDate,
    NaiveTime,
    NaiveDateTime,
    DateTime<Utc>,
    num_bigint::BigInt,
    BigDecimal,
    isize,
    usize,
    Duration,
    Url,
    HashMap<String, String>,
    serde_json::Value,
);
