// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A formatter used to test incremental display writes.

use std::fmt::{self, Write};

/// Writer that counts formatting calls and can fail at a selected call.
pub(crate) struct CountingWriter {
    /// Number of completed or attempted writes.
    write_count: usize,
    /// Optional one-based write index that returns [`fmt::Error`].
    fail_on_write: Option<usize>,
}

impl CountingWriter {
    /// Creates a writer that optionally fails on `fail_on_write`.
    ///
    /// # Parameters
    ///
    /// * `fail_on_write` - Optional one-based write index to fail.
    ///
    /// # Returns
    ///
    /// A new counting writer.
    #[inline]
    pub(crate) const fn new(fail_on_write: Option<usize>) -> Self {
        Self {
            write_count: 0,
            fail_on_write,
        }
    }

    /// Returns the number of completed or attempted writes.
    ///
    /// # Returns
    ///
    /// The observed write count.
    #[inline(always)]
    pub(crate) const fn write_count(&self) -> usize {
        self.write_count
    }
}

impl Write for CountingWriter {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        self.write_count += 1;
        if self.fail_on_write == Some(self.write_count) {
            return Err(fmt::Error);
        }
        Ok(())
    }
}
