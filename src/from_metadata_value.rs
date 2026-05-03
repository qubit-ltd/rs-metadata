/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Provides conversion from metadata values into supported Rust values.

use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use qubit_value::{Value, ValueResult};
use url::Url;

/// Converts metadata backing [`Value`] instances into supported Rust types.
///
/// This trait is implemented for the same concrete output types accepted by
/// [`Value::to`]. It keeps metadata accessors generic without depending on
/// `qubit-value` internal conversion traits.
pub trait FromMetadataValue: Sized {
    /// Converts `value` into `Self`.
    ///
    /// # Errors
    ///
    /// Returns a [`qubit_value::ValueError`] when the stored value is absent,
    /// unsupported, or cannot be converted to `Self`.
    fn from_metadata_value(value: &Value) -> ValueResult<Self>;
}

macro_rules! impl_from_metadata_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl FromMetadataValue for $type {
                #[inline]
                fn from_metadata_value(value: &Value) -> ValueResult<Self> {
                    value.to::<$type>()
                }
            }
        )+
    };
}

impl_from_metadata_value!(
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
