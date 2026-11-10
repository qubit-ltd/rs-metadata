// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared filter-limit helpers for expression wire decoding.

use std::cell::RefCell;
use std::rc::Rc;

use qubit_budget::InsufficientBudgetError;

use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::MetadataError;

/// Converts shared budget facts to the established metadata filter error.
///
/// # Parameters
///
/// * `error` - Failed cumulative-budget request, including its resource, limit,
///   remaining capacity, and requested amount.
///
/// # Returns
///
/// A filter-limit error whose observed value is the consumed amount plus the
/// failed request, using saturating arithmetic.
pub(in crate::filter::wire) fn filter_limit_error(
    error: InsufficientBudgetError<FilterLimitKind, usize>,
) -> MetadataError {
    let InsufficientBudgetError {
        resource,
        limit,
        remaining,
        requested,
    } = error;
    MetadataError::FilterLimitExceeded {
        kind: resource,
        value: limit.saturating_sub(remaining).saturating_add(requested),
        maximum: limit,
    }
}

/// Stores the first receiver-limit error produced during one decode call.
///
/// # Parameters
///
/// * `error_slot` - Shared slot retained by the decode operation for reporting
///   a structured receiver-limit failure after deserialization fails.
/// * `error` - Structured metadata error to preserve if no earlier limit error
///   has been recorded.
pub(in crate::filter::wire) fn capture_filter_error(
    error_slot: &Rc<RefCell<Option<MetadataError>>>,
    error: MetadataError,
) {
    let mut captured = error_slot.borrow_mut();
    if captured.is_none() {
        *captured = Some(error);
    }
}

/// Checks one decoded key against receiver filter limits.
///
/// # Parameters
///
/// * `key` - Decoded metadata key whose UTF-8 byte length is checked.
/// * `receiver_limits` - Limits selected by the receiving side of decoding.
/// * `error_slot` - Shared slot in which the structured limit error is saved.
///
/// # Errors
///
/// Returns the structured filter-limit error when the key exceeds the
/// receiver's maximum key length.
///
/// # Returns
///
/// Returns `Ok(())` when the key fits the receiver limit.
pub(in crate::filter::wire) fn check_key(
    key: &str,
    receiver_limits: FilterLimits,
    error_slot: &Rc<RefCell<Option<MetadataError>>>,
) -> Result<(), MetadataError> {
    if key.len() > receiver_limits.max_key_bytes() {
        let error = MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::KeyBytes,
            value: key.len(),
            maximum: receiver_limits.max_key_bytes(),
        };
        capture_filter_error(error_slot, error.clone());
        return Err(error);
    }
    Ok(())
}

/// Takes a parsed field value or reports that its wire field was missing.
///
/// # Type Parameters
///
/// * `T` - Type of the parsed field value.
///
/// # Parameters
///
/// * `value` - Optional parsed value for a field required by the expression
///   kind.
/// * `field` - Wire field name included in the missing-field message.
///
/// # Errors
///
/// Returns a message naming `field` when `value` is `None`.
///
/// # Returns
///
/// Returns the parsed field value when present.
#[inline]
pub(super) fn required<T>(value: Option<T>, field: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("missing field `{field}`"))
}

/// Ensures a wire field not allowed by the selected expression kind is absent.
///
/// # Type Parameters
///
/// * `T` - Type of the parsed field value.
///
/// # Parameters
///
/// * `value` - Optional parsed value for a field excluded by the expression
///   kind.
/// * `field` - Wire field name included in the unknown-field message.
///
/// # Errors
///
/// Returns a message naming `field` when it is present for this expression
/// kind.
///
/// # Returns
///
/// Returns `Ok(())` when the field is absent.
#[inline]
pub(super) fn ensure_absent<T>(value: Option<T>, field: &str) -> Result<(), String> {
    if value.is_some() {
        Err(format!("unknown field `{field}` for this expression kind"))
    } else {
        Ok(())
    }
}
