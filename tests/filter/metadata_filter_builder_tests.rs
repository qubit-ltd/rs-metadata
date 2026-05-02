/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Unit tests for [`qubit_metadata::MetadataFilterBuilder`] default behavior.
use qubit_metadata::MetadataFilterBuilder;

#[test]
fn builder_default_builds_match_all_filter() {
    let filter = MetadataFilterBuilder::default().build().unwrap();
    assert!(filter.matches(&qubit_metadata::Metadata::new()));
}
