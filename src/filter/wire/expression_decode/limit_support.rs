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
use serde::de;

use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::MetadataError;

/// Converts shared budget facts to the established metadata filter error.
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
pub(in crate::filter::wire) fn check_key<E>(
    key: &str,
    receiver_limits: FilterLimits,
    error_slot: &Rc<RefCell<Option<MetadataError>>>,
) -> Result<(), E>
where
    E: de::Error,
{
    if key.len() > receiver_limits.max_key_bytes() {
        let error = MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::KeyBytes,
            value: key.len(),
            maximum: receiver_limits.max_key_bytes(),
        };
        capture_filter_error(error_slot, error.clone());
        return Err(E::custom(error));
    }
    Ok(())
}

/// Returns a required field or a missing-field message.
pub(super) fn required<T>(value: Option<T>, field: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("missing field `{field}`"))
}

/// Rejects a field that is not valid for the selected expression kind.
pub(super) fn ensure_absent<T>(value: Option<T>, field: &str) -> Result<(), String> {
    if value.is_some() {
        Err(format!("unknown field `{field}` for this expression kind"))
    } else {
        Ok(())
    }
}
