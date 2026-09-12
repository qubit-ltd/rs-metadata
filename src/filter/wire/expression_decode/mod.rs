// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal modules for incremental V1 filter-expression decoding.

mod expression_element_seed;
mod expression_fields;
mod expression_kind;
mod expression_sequence_seed;
mod expression_sequence_visitor;
mod expression_visitor;
mod limit_support;
mod value_element_seed;
mod value_sequence_seed;
mod value_sequence_visitor;

pub(in crate::filter::wire) use expression_visitor::ExpressionVisitor;
pub(in crate::filter::wire) use limit_support::capture_filter_error;
pub(in crate::filter::wire) use limit_support::filter_limit_error;
