// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Core wire-envelope test modules.

#[cfg(feature = "schema")]
mod metadata_schema_wire_v1_tests;
#[cfg(all(feature = "json", feature = "schema"))]
mod metadata_wire_limits_tests;
mod metadata_wire_v1_tests;
mod strict_string_map_tests;
