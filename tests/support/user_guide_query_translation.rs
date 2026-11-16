// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;

pub fn main() -> qubit_metadata::MetadataResult<()> {
    let metadata = Metadata::new();
    let comparison = MetadataFilter::builder()
        .expression(
            FilterExpression::builder()
                .eq("tenant", "acme")
                .not()
                .build()?,
        )
        .build()?;
    let absent = MetadataFilter::builder()
        .expression(FilterExpression::builder().not_exists("tenant").build()?)
        .build()?;
    assert!(!comparison.matches(&metadata));
    assert!(absent.matches(&metadata));
    Ok(())
}
