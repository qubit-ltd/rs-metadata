// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A downstream conversion target used by metadata tests.

use qubit_datatype::ConversionContext;
use qubit_datatype::DataConversionError;
use qubit_datatype::DataConversionTarget;
use qubit_datatype::DataConverter;
use qubit_datatype::DataType;
use qubit_datatype::DataTypeOf;

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
        context: &mut ConversionContext<'_, '_>,
    ) -> Result<Self, DataConversionError> {
        context.delegate::<u16>(source).map(Self)
    }
}
