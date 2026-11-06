// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal filter implementation types.

mod expression_children;
mod filter_expression_node;
mod filter_match_options_visitor;
mod match_outcome;

pub(crate) use expression_children::ExpressionChildren;
pub(crate) use filter_expression_node::FilterExpressionNode;
pub(crate) use filter_match_options_visitor::deserialize_filter_match_options;
pub(crate) use match_outcome::MatchOutcome;
