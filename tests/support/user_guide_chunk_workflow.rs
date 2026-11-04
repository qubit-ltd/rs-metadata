// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_datatype::DataType;
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataSchema;

pub trait ChunkStore {
    fn insert(&mut self, metadata: Metadata) -> Result<(), Box<dyn std::error::Error>>;
}

fn ingest(
    store: &mut impl ChunkStore,
    schema: &MetadataSchema,
    metadata: Metadata,
) -> Result<(), Box<dyn std::error::Error>> {
    schema.validate(&metadata)?;
    store.insert(metadata)
}

#[derive(Default)]
struct RecordingStore(Vec<Metadata>);

impl ChunkStore for RecordingStore {
    fn insert(&mut self, metadata: Metadata) -> Result<(), Box<dyn std::error::Error>> {
        self.0.push(metadata);
        Ok(())
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = MetadataSchema::builder()
        .required("tenant_id", DataType::String)
        .required("chunk_index", DataType::Int64)
        .optional("language", DataType::String)
        .build()?;
    let record = Metadata::new()
        .with("tenant_id", "acme")
        .with("chunk_index", 3_i64)
        .with("language", "en");
    let mut store = RecordingStore::default();
    ingest(&mut store, &schema, record.clone())?;
    assert_eq!(store.0.len(), 1);
    assert!(ingest(&mut store, &schema, Metadata::new().with("tenant_id", "acme")).is_err());
    assert_eq!(store.0.len(), 1);

    let filter = MetadataFilter::builder()
        .expression(
            FilterExpression::builder()
                .eq("tenant_id", "acme")
                .ge("chunk_index", 2_i64)
                .build()?,
        )
        .build_checked(&schema)?;
    assert!(filter.matches(&record));
    assert!(!filter.matches(&Metadata::new().with("tenant_id", "other").with("chunk_index", 3_i64)));

    let invalid_query = FilterExpression::builder().ge("chunk_index", "two").build()?;
    assert!(
        MetadataFilter::builder()
            .expression(invalid_query)
            .build_checked(&schema)
            .is_err()
    );
    Ok(())
}
