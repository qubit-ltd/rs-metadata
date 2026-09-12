// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seed that bounds a membership-value sequence.

use std::cell::RefCell;
use std::rc::Rc;

use qubit_value::ValueWirePayloadV1;
use serde::de::DeserializeSeed;
use serde::de::Deserializer;

use super::value_sequence_visitor::ValueSequenceVisitor;
use crate::FilterLimits;
use crate::MetadataError;

/// Seed that bounds a membership-value sequence.
pub(in crate::filter::wire) struct ValueSequenceSeed {
    /// Receiver-side membership limit.
    pub(in crate::filter::wire) receiver_limits: FilterLimits,
    /// First structured limit error.
    pub(in crate::filter::wire) error_slot: Rc<RefCell<Option<MetadataError>>>,
}

impl<'de> DeserializeSeed<'de> for ValueSequenceSeed {
    type Value = Vec<ValueWirePayloadV1>;

    /// Decodes membership values with receiver-controlled bounds.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ValueSequenceVisitor {
            receiver_limits: self.receiver_limits,
            error_slot: self.error_slot,
        })
    }
}
