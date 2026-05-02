/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`MetadataFilterWire`] and wire version.
use serde::{Deserialize, Serialize};

use super::super::metadata_filter::MetadataFilter;
use super::filter_expr_wire::FilterExprWire;
use crate::FilterMatchOptions;

pub(crate) const METADATA_FILTER_WIRE_VERSION: u8 = 1;

#[inline]
const fn metadata_filter_wire_version() -> u8 {
    METADATA_FILTER_WIRE_VERSION
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataFilterWire {
    #[serde(default = "metadata_filter_wire_version")]
    version: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) expr: Option<FilterExprWire>,
    #[serde(default)]
    pub(crate) options: FilterMatchOptions,
}

impl MetadataFilterWire {
    pub(crate) fn into_filter(self) -> Result<MetadataFilter, String> {
        if self.version != METADATA_FILTER_WIRE_VERSION {
            return Err(format!(
                "unsupported MetadataFilter wire format version {}; expected {}",
                self.version, METADATA_FILTER_WIRE_VERSION
            ));
        }
        let expr = self.expr.map(FilterExprWire::into_expr).transpose()?;
        Ok(MetadataFilter::new(expr, self.options))
    }
}

impl From<&MetadataFilter> for MetadataFilterWire {
    fn from(filter: &MetadataFilter) -> Self {
        Self {
            version: METADATA_FILTER_WIRE_VERSION,
            expr: filter.expr.as_ref().map(FilterExprWire::from),
            options: filter.options,
        }
    }
}
