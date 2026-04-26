/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Wire-format support for metadata filters.

mod condition;
mod expr;
mod input;
mod legacy_expr;
mod legacy_metadata_filter;
mod metadata_filter;

pub(crate) use input::MetadataFilterInput;
pub(crate) use metadata_filter::MetadataFilterWire;
