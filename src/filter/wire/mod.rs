/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Wire-format support for metadata filters.

mod condition_wire;
mod filter_expr_wire;
mod metadata_filter_wire;

pub(crate) use condition_wire::ConditionWire;
pub(crate) use metadata_filter_wire::MetadataFilterWire;
