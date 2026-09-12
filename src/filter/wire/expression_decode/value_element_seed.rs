// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seed that checks one membership item before reading its payload.

use std::cell::RefCell;
use std::rc::Rc;

use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWirePayloadV1Seed;
use serde::de::DeserializeSeed;
use serde::de::Deserializer;
use serde::de::Error as _;

use super::limit_support::capture_filter_error;
use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::MetadataError;

/// Seed that checks one membership item before reading its payload.
pub(in crate::filter::wire) struct ValueElementSeed {
    /// Receiver-side membership limit.
    pub(in crate::filter::wire) receiver_limits: FilterLimits,
    /// Next membership position being decoded.
    pub(in crate::filter::wire) next_len: usize,
    /// First structured limit error.
    pub(in crate::filter::wire) error_slot: Rc<RefCell<Option<MetadataError>>>,
}

impl<'de> DeserializeSeed<'de> for ValueElementSeed {
    type Value = ValueWirePayloadV1;

    /// Decodes one membership payload after charging its collection position.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        if self.next_len > self.receiver_limits.max_set_values() {
            let error = MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::SetValues,
                value: self.next_len,
                maximum: self.receiver_limits.max_set_values(),
            };
            capture_filter_error(&self.error_slot, error.clone());
            return Err(D::Error::custom(error));
        }
        ValueWirePayloadV1Seed::new().deserialize(deserializer)
    }
}
