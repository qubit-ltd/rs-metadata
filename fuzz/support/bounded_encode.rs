// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzz-target access to the shared bounded-encoding oracle.

#[path = "../../tests/support/bounded_encode.rs"]
mod shared;

pub use shared::caps;
pub use shared::check;
