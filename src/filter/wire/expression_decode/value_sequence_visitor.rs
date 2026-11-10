// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Visitor that decodes membership payloads under the receiver's value limit.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use qubit_value::ValueWirePayloadV1;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::value_element_seed::ValueElementSeed;
use crate::FilterLimits;
use crate::MetadataError;

/// Decodes each membership payload while carrying receiver-side resource
/// policy.
pub(in crate::filter::wire) struct ValueSequenceVisitor {
    /// Limits used to reject an element before decoding it when the sequence is
    /// full.
    pub(in crate::filter::wire) receiver_limits: FilterLimits,
    /// Shared slot preserving the first structured limit error for the caller.
    pub(in crate::filter::wire) error_slot: Rc<RefCell<Option<MetadataError>>>,
}

impl<'de> Visitor<'de> for ValueSequenceVisitor {
    /// The decoded sequence of V1 value wire payloads.
    type Value = Vec<ValueWirePayloadV1>;

    /// Describes the sequence shape expected by this filter decoder.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination for the human-readable expected shape.
    ///
    /// # Returns
    ///
    /// The formatter's result after writing the expected sequence description.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a sequence of scalar V1 value payloads")
    }

    /// Decodes membership payloads one by one, checking each position before
    /// its payload is read and avoiding capacity allocation beyond the limit.
    ///
    /// # Type Parameters
    ///
    /// * `A` - Serde access object that supplies the sequence elements.
    ///
    /// # Parameters
    ///
    /// * `sequence` - Serde access object for the encoded membership sequence.
    ///
    /// # Errors
    ///
    /// Returns the deserializer's error if an element is malformed or exceeds
    /// the configured membership limit. The first structured limit error is
    /// retained in `error_slot` for the enclosing decoder.
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
