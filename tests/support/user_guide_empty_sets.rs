// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_datatype::DataType;
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
use qubit_value::Value;

pub fn main() -> qubit_metadata::MetadataResult<()> {
    let included = MetadataFilter::builder()
        .expression(FilterExpression::builder().in_set("key", std::iter::empty::<i64>()).build()?)
        .build()?;
    let excluded = MetadataFilter::builder()
        .expression(FilterExpression::builder().not_in_set("key", std::iter::empty::<i64>()).build()?)
        .build()?;
    let concrete = Metadata::new().with("key", 1_i64);
    assert!(!included.matches(&concrete));
    assert!(excluded.matches(&concrete));
    assert!(!excluded.matches(&Metadata::new()));
    assert!(!excluded.matches(&Metadata::new().with("key", Value::Unset(DataType::Int64))));
    Ok(())
}
