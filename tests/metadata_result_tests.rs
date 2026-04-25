/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use qubit_metadata::MetadataResult;

#[test]
fn metadata_result_alias_compiles() {
    let value: MetadataResult<i32> = Ok(7);
    assert_eq!(value.unwrap(), 7);
}
