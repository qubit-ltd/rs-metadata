/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_metadata::{Metadata, MetadataFilter};

#[test]
fn public_exports_are_usable() {
    let meta = Metadata::new().with("k", "v");
    let filter = MetadataFilter::builder().eq("k", "v").build().unwrap();
    assert!(filter.matches(&meta));
}
