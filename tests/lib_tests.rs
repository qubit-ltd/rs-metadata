/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use qubit_metadata::{Metadata, MetadataFilter};

#[test]
fn public_exports_are_usable() {
    let meta = Metadata::new().with("k", "v");
    let filter = MetadataFilter::builder().eq("k", "v").build();
    assert!(filter.matches(&meta));
}
