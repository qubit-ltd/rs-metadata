// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Integration coverage for optional scalar value families.

#[cfg(feature = "chrono")]
mod chrono_values {
    use chrono::NaiveDate;
    use chrono::NaiveDateTime;
    use chrono::NaiveTime;
    use qubit_datatype::DataType;
    use qubit_metadata::Metadata;

    #[test]
    fn strict_reads_preserve_chrono_types() {
        let date = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let time = NaiveTime::from_hms_opt(12, 34, 56).unwrap();
        let datetime = NaiveDateTime::new(date, time);
        let metadata = Metadata::new()
            .with("date", date)
            .with("time", time)
            .with("datetime", datetime);

        assert_eq!(metadata.get::<NaiveDate>("date").unwrap(), date);
        assert_eq!(metadata.get::<NaiveTime>("time").unwrap(), time);
        assert_eq!(metadata.get::<NaiveDateTime>("datetime").unwrap(), datetime);
        assert!(metadata.get::<String>("date").is_err());
        assert_eq!(metadata.get_raw("date").unwrap().data_type(), DataType::Date);

        #[cfg(feature = "json")]
        {
            use qubit_budget::BudgetError;
            use qubit_budget::ResourceLimit;
            use qubit_budget::json::JsonEncodeLimits;
            use qubit_budget::json::JsonResource;
            use qubit_metadata::MetadataWireEncodeError;

            let bytes = metadata.to_json_vec().unwrap();
            assert_eq!(serde_json::to_vec(&metadata).unwrap(), bytes);
            let decoded = Metadata::decode_json_slice(&bytes).unwrap();
            assert_eq!(decoded, metadata);
            assert_eq!(decoded.get_raw("date").unwrap().data_type(), DataType::Date);
            assert_eq!(decoded.get_raw("time").unwrap().data_type(), DataType::Time);
            assert_eq!(decoded.get_raw("datetime").unwrap().data_type(), DataType::DateTime);
            let limits = JsonEncodeLimits::builder()
                .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
                .build();
            assert!(matches!(
                metadata.to_json_vec_with_limits(limits),
                Err(MetadataWireEncodeError::Budget(BudgetError::Insufficient {
                    resource: JsonResource::OutputBytes,
                    ..
                }))
            ));
        }

        #[cfg(feature = "schema")]
        {
            let date_schema = qubit_metadata::MetadataSchema::builder()
                .required("date", DataType::Date)
                .build()
                .unwrap();
            assert!(date_schema.validate(&Metadata::new().with("date", date)).is_ok());
            let wrong_schema = qubit_metadata::MetadataSchema::builder()
                .required("date", DataType::String)
                .build()
                .unwrap();
            assert!(wrong_schema.validate(&Metadata::new().with("date", date)).is_err());
        }
    }
}

#[cfg(feature = "big-integer")]
mod big_integer_values {
    use std::str::FromStr;

    use num_bigint::BigInt;
    use qubit_datatype::DataType;
    use qubit_metadata::Metadata;

    #[test]
    fn strict_reads_preserve_wide_integers() {
        let positive = BigInt::from_str("170141183460469231731687303715884105728").unwrap();
        let negative = BigInt::from_str("-170141183460469231731687303715884105729").unwrap();
        let metadata = Metadata::new()
            .with("positive", positive.clone())
            .with("negative", negative.clone());

        assert_eq!(metadata.get::<BigInt>("positive").unwrap(), positive);
        assert_eq!(metadata.get::<BigInt>("negative").unwrap(), negative);
        assert!(metadata.get::<i128>("positive").is_err());
        assert_eq!(metadata.get_raw("positive").unwrap().data_type(), DataType::BigInteger);

        #[cfg(feature = "json")]
        {
            use qubit_budget::BudgetError;
            use qubit_budget::ResourceLimit;
            use qubit_budget::json::JsonEncodeLimits;
            use qubit_budget::json::JsonResource;
            use qubit_metadata::MetadataWireEncodeError;

            let bytes = metadata.to_json_vec().unwrap();
            assert_eq!(serde_json::to_vec(&metadata).unwrap(), bytes);
            let decoded = Metadata::decode_json_slice(&bytes).unwrap();
            assert_eq!(decoded, metadata);
            assert_eq!(decoded.get_raw("positive").unwrap().data_type(), DataType::BigInteger);
            assert_eq!(decoded.get::<BigInt>("negative").unwrap(), negative);
            let limits = JsonEncodeLimits::builder()
                .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
                .build();
            assert!(matches!(
                metadata.to_json_vec_with_limits(limits),
                Err(MetadataWireEncodeError::Budget(BudgetError::Insufficient {
                    resource: JsonResource::OutputBytes,
                    ..
                }))
            ));
        }

        #[cfg(feature = "schema")]
        {
            let schema = qubit_metadata::MetadataSchema::builder()
                .required("positive", DataType::BigInteger)
                .build()
                .unwrap();
            let checked = Metadata::new()
                .with_checked(&schema, "positive", positive.clone())
                .unwrap();
            assert_eq!(checked.get::<BigInt>("positive").unwrap(), positive);
            let mismatch = qubit_metadata::MetadataSchema::builder()
                .required("positive", DataType::BigDecimal)
                .build()
                .unwrap();
            assert!(
                Metadata::new()
                    .with_checked(&mismatch, "positive", positive.clone())
                    .is_err()
            );
        }

        #[cfg(feature = "filter")]
        {
            let expression = qubit_metadata::FilterExpression::builder()
                .eq("positive", positive.clone())
                .build()
                .unwrap();
            let filter = qubit_metadata::MetadataFilter::builder()
                .expression(expression)
                .build()
                .unwrap();
            assert!(filter.matches(&Metadata::new().with("positive", positive)));
            assert!(!filter.matches(&Metadata::new().with("positive", BigInt::from(7))));
        }
    }
}

#[cfg(feature = "big-decimal")]
mod big_decimal_values {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use qubit_datatype::DataType;
    use qubit_metadata::Metadata;

    #[test]
    fn strict_reads_preserve_wide_decimals() {
        let decimal = BigDecimal::from_str("9007199254740993.1234567890123456789").unwrap();
        let metadata = Metadata::new().with("decimal", decimal.clone());

        assert_eq!(metadata.get::<BigDecimal>("decimal").unwrap(), decimal);
        assert!(metadata.get::<f64>("decimal").is_err());
        assert_eq!(metadata.get_raw("decimal").unwrap().data_type(), DataType::BigDecimal);

        #[cfg(feature = "json")]
        {
            use qubit_budget::BudgetError;
            use qubit_budget::ResourceLimit;
            use qubit_budget::json::JsonEncodeLimits;
            use qubit_budget::json::JsonResource;
            use qubit_metadata::MetadataWireEncodeError;

            let bytes = metadata.to_json_vec().unwrap();
            let decoded = Metadata::decode_json_slice(&bytes).unwrap();
            assert_eq!(decoded, metadata);
            assert_eq!(decoded.get::<BigDecimal>("decimal").unwrap(), decimal);
            assert_eq!(decoded.get_raw("decimal").unwrap().data_type(), DataType::BigDecimal);
            let limits = JsonEncodeLimits::builder()
                .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
                .build();
            assert!(matches!(
                metadata.to_json_vec_with_limits(limits),
                Err(MetadataWireEncodeError::Budget(BudgetError::Insufficient {
                    resource: JsonResource::OutputBytes,
                    ..
                }))
            ));
        }

        #[cfg(feature = "schema")]
        {
            let schema = qubit_metadata::MetadataSchema::builder()
                .required("decimal", DataType::BigDecimal)
                .build()
                .unwrap();
            let checked = Metadata::new()
                .with_checked(&schema, "decimal", decimal.clone())
                .unwrap();
            assert_eq!(checked.get::<BigDecimal>("decimal").unwrap(), decimal);
            let mismatch = qubit_metadata::MetadataSchema::builder()
                .required("decimal", DataType::BigInteger)
                .build()
                .unwrap();
            assert!(
                Metadata::new()
                    .with_checked(&mismatch, "decimal", decimal.clone())
                    .is_err()
            );
        }

        #[cfg(feature = "filter")]
        {
            let expression = qubit_metadata::FilterExpression::builder()
                .eq("decimal", decimal.clone())
                .build()
                .unwrap();
            let filter = qubit_metadata::MetadataFilter::builder()
                .expression(expression)
                .build()
                .unwrap();
            assert!(filter.matches(&Metadata::new().with("decimal", decimal)));
            assert!(!filter.matches(&Metadata::new().with("decimal", BigDecimal::from(2))));
        }
    }
}

#[cfg(feature = "url")]
mod url_values {
    use qubit_datatype::DataType;
    use qubit_metadata::Metadata;
    use url::Url;

    #[test]
    fn strict_reads_preserve_url_values() {
        let url = Url::parse("https://example.com/café/items?tag=rust&mode=full").unwrap();
        let metadata = Metadata::new().with("url", url.clone());

        assert_eq!(metadata.get::<Url>("url").unwrap(), url);
        assert!(metadata.get::<String>("url").is_err());
        assert_eq!(metadata.get_raw("url").unwrap().data_type(), DataType::Url);

        #[cfg(feature = "json")]
        {
            use qubit_budget::BudgetError;
            use qubit_budget::ResourceLimit;
            use qubit_budget::json::JsonEncodeLimits;
            use qubit_budget::json::JsonResource;
            use qubit_metadata::MetadataWireEncodeError;

            let bytes = metadata.to_json_vec().unwrap();
            let decoded = Metadata::decode_json_slice(&bytes).unwrap();
            assert_eq!(decoded, metadata);
            assert_eq!(decoded.get::<Url>("url").unwrap(), url);
            assert_eq!(decoded.get_raw("url").unwrap().data_type(), DataType::Url);
            let limits = JsonEncodeLimits::builder()
                .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
                .build();
            assert!(matches!(
                metadata.to_json_vec_with_limits(limits),
                Err(MetadataWireEncodeError::Budget(BudgetError::Insufficient {
                    resource: JsonResource::OutputBytes,
                    ..
                }))
            ));
        }

        #[cfg(feature = "schema")]
        {
            let schema = qubit_metadata::MetadataSchema::builder()
                .required("url", DataType::Url)
                .build()
                .unwrap();
            assert!(schema.validate(&Metadata::new().with("url", url.clone())).is_ok());
            let wrong_schema = qubit_metadata::MetadataSchema::builder()
                .required("url", DataType::String)
                .build()
                .unwrap();
            assert!(wrong_schema.validate(&Metadata::new().with("url", url)).is_err());
        }
    }
}
