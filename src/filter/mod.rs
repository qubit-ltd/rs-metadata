/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Composable metadata filters: [`MetadataFilter`], wire types, and builder.
mod condition;
mod filter_expr;
mod filter_match_options;
mod metadata_filter;
mod metadata_filter_builder;
mod missing_key_policy;
mod number_comparison_policy;
mod wire;

pub use condition::Condition;
pub use filter_match_options::FilterMatchOptions;
pub use metadata_filter::MetadataFilter;
pub use metadata_filter_builder::MetadataFilterBuilder;
pub use missing_key_policy::MissingKeyPolicy;
pub use number_comparison_policy::NumberComparisonPolicy;
