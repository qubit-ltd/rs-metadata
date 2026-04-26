/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! [`LegacyMetadataFilterWire`].
use serde::Deserialize;

use super::legacy_expr::LegacyFilterExpr;
use super::super::metadata_filter::MetadataFilter;
use crate::FilterMatchOptions;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LegacyMetadataFilterWire {
    #[serde(default)]
    pub(crate) expr: Option<LegacyFilterExpr>,
    #[serde(default)]
    pub(crate) options: FilterMatchOptions,
}

impl LegacyMetadataFilterWire {
    pub(crate) fn into_filter(self) -> MetadataFilter {
        MetadataFilter::new(
            self.expr.map(LegacyFilterExpr::into_expr),
            self.options,
        )
    }
}
