// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Filter-limits wire tests through the public filter envelope.

use qubit_metadata::{
    FilterLimits,
    MetadataFilter,
};
use serde_json::json;

#[test]
fn test_filter_limits_wire_contains_all_bounds_and_rejects_unknown_fields() {
    let encoded = serde_json::to_value(MetadataFilter::all()).unwrap();
    assert_eq!(
        encoded["limits"]["max_depth"],
        json!(FilterLimits::MAX.max_depth())
    );
    assert_eq!(
        encoded["limits"]["max_nodes"],
        json!(FilterLimits::MAX.max_nodes())
    );
    assert_eq!(
        encoded["limits"]["max_set_values"],
        json!(FilterLimits::MAX.max_set_values())
    );
    assert_eq!(
        encoded["limits"]["max_key_bytes"],
        json!(FilterLimits::MAX.max_key_bytes())
    );

    let mut invalid = encoded;
    invalid["limits"]["extra"] = json!(1);
    assert!(serde_json::from_value::<MetadataFilter>(invalid).is_err());
}
