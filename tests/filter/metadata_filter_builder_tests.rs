/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Unit tests for [`qubit_metadata::MetadataFilterBuilder`] default behavior.
use qubit_metadata::MetadataFilterBuilder;

#[test]
fn builder_default_builds_match_all_filter() {
    let filter = MetadataFilterBuilder::default().build();
    assert!(filter.matches(&qubit_metadata::Metadata::new()));
}
