/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! [`MetadataFilterInput`].
use serde::Deserialize;

use super::legacy_metadata_filter::LegacyMetadataFilterWire;
use super::metadata_filter::MetadataFilterWire;
use super::super::metadata_filter::MetadataFilter;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub(crate) enum MetadataFilterInput {
    Wire(MetadataFilterWire),
    Legacy(LegacyMetadataFilterWire),
}

impl MetadataFilterInput {
    pub(crate) fn into_filter(self) -> Result<MetadataFilter, String> {
        match self {
            Self::Wire(wire) => wire.into_filter(),
            Self::Legacy(legacy) => Ok(legacy.into_filter()),
        }
    }
}
