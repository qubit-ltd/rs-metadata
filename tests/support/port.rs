// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A downstream conversion target used by metadata tests.

use qubit_datatype::{
    ConversionSession, DataConversionError, DataConversionTarget, DataConverter, DataType,
    DataTypeOf,
};

/// Strongly typed port number used as a downstream conversion target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Port(
    /// Stored port number.
    pub(crate) u16,
);

impl DataTypeOf for Port {
    const DATA_TYPE: DataType = DataType::UInt16;
}

impl DataConversionTarget for Port {
    fn convert_from(
        source: &DataConverter<'_>,
        session: &mut ConversionSession<'_>,
    ) -> Result<Self, DataConversionError> {
        session.delegate::<u16>(source).map(Self)
    }
}
