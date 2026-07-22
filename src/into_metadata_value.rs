// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provides conversion from supported Rust values into metadata values.

use std::collections::HashMap;
use std::time::Duration;

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "chrono")]
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use qubit_value::Value;
#[cfg(feature = "url")]
use url::Url;

/// Converts supported Rust values into the metadata backing [`Value`] type.
///
/// This trait is implemented for the concrete input types accepted by
/// [`crate::Metadata`]. It keeps metadata's generic public API constrained to
/// its deliberate domain allowlist instead of accepting every future
/// conversion into [`Value`].
pub trait IntoMetadataValue {
    /// Converts this value into a [`Value`] without changing its concrete data
    /// type.
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
                #[inline(always)]
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
    Duration,
    HashMap<String, String>,
);

#[cfg(feature = "chrono")]
impl_into_metadata_value!(NaiveDate, NaiveTime, NaiveDateTime, DateTime<Utc>,);

#[cfg(feature = "big-integer")]
impl_into_metadata_value!(num_bigint::BigInt);

#[cfg(feature = "big-decimal")]
impl_into_metadata_value!(BigDecimal);

#[cfg(feature = "url")]
impl_into_metadata_value!(Url);

#[cfg(feature = "json")]
impl_into_metadata_value!(serde_json::Value);
