// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal filter implementation types.

mod filter_expression_node;
mod match_outcome;

pub(crate) use filter_expression_node::FilterExpressionNode;
pub(crate) use match_outcome::MatchOutcome;
