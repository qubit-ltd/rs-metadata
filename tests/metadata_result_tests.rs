/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_metadata::MetadataResult;

#[test]
fn metadata_result_alias_compiles() {
    let value: MetadataResult<i32> = Ok(7);
    assert!(matches!(value, Ok(7)));
}
