// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Visitor for a bounded membership-value sequence.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use qubit_value::ValueWirePayloadV1;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::value_element_seed::ValueElementSeed;
use crate::FilterLimits;
use crate::MetadataError;

/// Visitor for a bounded membership-value sequence.
pub(in crate::filter::wire) struct ValueSequenceVisitor {
    /// Receiver-side membership limit.
    pub(in crate::filter::wire) receiver_limits: FilterLimits,
    /// First structured limit error.
    pub(in crate::filter::wire) error_slot: Rc<RefCell<Option<MetadataError>>>,
}

impl<'de> Visitor<'de> for ValueSequenceVisitor {
    type Value = Vec<ValueWirePayloadV1>;

    /// Describes a sequence of scalar V1 payloads.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a sequence of scalar V1 value payloads")
    }

    /// Decodes membership values one by one and bounds vector growth.
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let maximum = self.receiver_limits.max_set_values();
        let capacity = sequence.size_hint().unwrap_or(0).min(maximum);
        let mut values = Vec::with_capacity(capacity);
        while let Some(value) = sequence.next_element_seed(ValueElementSeed {
            receiver_limits: self.receiver_limits,
            next_len: values.len().saturating_add(1),
            error_slot: Rc::clone(&self.error_slot),
        })? {
            values.push(value);
        }
        Ok(values)
    }
}
