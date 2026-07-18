// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Composable metadata filters: [`MetadataFilter`], wire types, and builder.
mod condition;
mod filter_expression;
mod filter_expression_view;
mod filter_limits;
mod filter_match_options;
mod internal;
mod metadata_filter;
mod metadata_filter_builder;
mod wire;

pub use condition::Condition;
pub use filter_expression::FilterExpression;
pub use filter_expression_view::FilterExpressionView;
pub use filter_limits::FilterLimits;
pub use filter_match_options::FilterMatchOptions;
pub use metadata_filter::MetadataFilter;
pub use metadata_filter_builder::MetadataFilterBuilder;
