/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Unit tests for [`qubit_metadata::MetadataFilter`] leaf predicate semantics.

use qubit_metadata::{
    Condition, FilterMatchOptions, Metadata, MetadataFilter, MissingKeyPolicy,
    NumberComparisonPolicy,
};

fn sample() -> Metadata {
    let mut m = Metadata::new();
    m.set("status", "active");
    m.set("score", 42_i64);
    m.set("ratio", 0.75_f64);
    m.set("verified", true);
    m.set("tag", "rust");
    m
}

#[test]
fn eq_matches_equal_string() {
    let f = MetadataFilter::builder().eq("status", "active");
    assert!(f.build().matches(&sample()));
}

#[test]
fn eq_does_not_match_different_string() {
    let f = MetadataFilter::builder().eq("status", "inactive");
    assert!(!f.build().matches(&sample()));
}

#[test]
fn eq_missing_key_does_not_match() {
    let f = MetadataFilter::builder().eq("missing", "x");
    assert!(!f.build().matches(&sample()));
}

#[test]
fn ne_matches_different_value() {
    let f = MetadataFilter::builder().ne("status", "inactive");
    assert!(f.build().matches(&sample()));
}

#[test]
fn ne_does_not_match_equal_value() {
    let f = MetadataFilter::builder().ne("status", "active");
    assert!(!f.build().matches(&sample()));
}

#[test]
fn ne_missing_key_matches_by_default() {
    let f = MetadataFilter::builder().ne("missing", "anything");
    assert!(f.build().matches(&sample()));
}

#[test]
fn ne_missing_key_respects_policy() {
    let f = MetadataFilter::builder().ne("missing", "anything");
    let match_options = FilterMatchOptions {
        missing_key_policy: MissingKeyPolicy::Match,
        number_comparison_policy: NumberComparisonPolicy::Conservative,
    };
    let no_match_options = FilterMatchOptions {
        missing_key_policy: MissingKeyPolicy::NoMatch,
        number_comparison_policy: NumberComparisonPolicy::Conservative,
    };
    assert!(
        f.clone()
            .build()
            .matches_with_options(&sample(), match_options)
    );
    assert!(!f.build().matches_with_options(&sample(), no_match_options));
}

#[test]
fn gt_integer() {
    assert!(
        MetadataFilter::builder()
            .gt("score", 10_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .gt("score", 42_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .gt("score", 100_i64)
            .build()
            .matches(&sample())
    );
}

#[test]
fn ge_integer() {
    assert!(
        MetadataFilter::builder()
            .ge("score", 42_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        MetadataFilter::builder()
            .ge("score", 10_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .ge("score", 43_i64)
            .build()
            .matches(&sample())
    );
}

#[test]
fn lt_integer() {
    assert!(
        MetadataFilter::builder()
            .lt("score", 100_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .lt("score", 42_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .lt("score", 10_i64)
            .build()
            .matches(&sample())
    );
}

#[test]
fn le_integer() {
    assert!(
        MetadataFilter::builder()
            .le("score", 42_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        MetadataFilter::builder()
            .le("score", 100_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .le("score", 41_i64)
            .build()
            .matches(&sample())
    );
}

#[test]
fn gt_string_lexicographic() {
    assert!(
        MetadataFilter::builder()
            .gt("status", "aaa")
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .gt("status", "zzz")
            .build()
            .matches(&sample())
    );
}

#[test]
fn range_filter_missing_key_does_not_match() {
    assert!(
        !MetadataFilter::builder()
            .gt("missing", 0_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .ge("missing", 0_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .lt("missing", 100_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .le("missing", 100_i64)
            .build()
            .matches(&sample())
    );
}

#[test]
fn range_filter_float_values() {
    assert!(
        MetadataFilter::builder()
            .gt("ratio", 0.5_f64)
            .build()
            .matches(&sample())
    );
    assert!(
        MetadataFilter::builder()
            .ge("ratio", 0.75_f64)
            .build()
            .matches(&sample())
    );
    assert!(
        MetadataFilter::builder()
            .lt("ratio", 1.0_f64)
            .build()
            .matches(&sample())
    );
    assert!(
        MetadataFilter::builder()
            .le("ratio", 0.75_f64)
            .build()
            .matches(&sample())
    );
}

#[test]
fn range_filter_u64_values() {
    let mut m = Metadata::new();
    m.set("count", 10_u64);

    assert!(
        MetadataFilter::builder()
            .gt("count", 9_u64)
            .build()
            .matches(&m)
    );
    assert!(
        MetadataFilter::builder()
            .ge("count", 10_u64)
            .build()
            .matches(&m)
    );
    assert!(
        MetadataFilter::builder()
            .lt("count", 11_u64)
            .build()
            .matches(&m)
    );
    assert!(
        MetadataFilter::builder()
            .le("count", 10_u64)
            .build()
            .matches(&m)
    );
}

#[test]
fn range_filter_mixed_signed_unsigned_values() {
    let mut a = Metadata::new();
    a.set("score", -1_i64);
    assert!(
        MetadataFilter::builder()
            .lt("score", 0_u64)
            .build()
            .matches(&a)
    );

    let mut b = Metadata::new();
    b.set("score", 5_u64);
    assert!(
        MetadataFilter::builder()
            .gt("score", 4_i64)
            .build()
            .matches(&b)
    );
}

#[test]
fn range_filter_mixed_signed_unsigned_with_huge_unsigned_values() {
    let huge = (i64::MAX as u64) + 10;

    let mut negative = Metadata::new();
    negative.set("score", -1_i64);
    assert!(
        MetadataFilter::builder()
            .lt("score", huge)
            .build()
            .matches(&negative)
    );

    let mut positive = Metadata::new();
    positive.set("score", 5_i64);
    assert!(
        MetadataFilter::builder()
            .lt("score", huge)
            .build()
            .matches(&positive)
    );

    let mut huge_unsigned = Metadata::new();
    huge_unsigned.set("score", huge);
    assert!(
        MetadataFilter::builder()
            .gt("score", i64::MAX)
            .build()
            .matches(&huge_unsigned)
    );
    assert!(
        MetadataFilter::builder()
            .gt("score", -1_i64)
            .build()
            .matches(&huge_unsigned)
    );
    assert!(
        MetadataFilter::builder()
            .gt("score", huge - 1)
            .build()
            .matches(&huge_unsigned)
    );
}

#[test]
fn range_filter_mixed_u64_and_f64() {
    let mut m = Metadata::new();
    m.set("count", 5_u64);

    assert!(
        MetadataFilter::builder()
            .gt("count", 4.5_f64)
            .build()
            .matches(&m)
    );
    assert!(
        !MetadataFilter::builder()
            .lt("count", 4.5_f64)
            .build()
            .matches(&m)
    );
}

#[test]
fn range_filter_large_integer_vs_float_precision_regression() {
    let mut m = Metadata::new();
    m.set("n", 9_007_199_254_740_993_i64);

    assert!(
        MetadataFilter::builder()
            .gt("n", 9_007_199_254_740_992_f64)
            .build()
            .matches(&m)
    );
    assert!(
        MetadataFilter::builder()
            .ge("n", 9_007_199_254_740_992_f64)
            .build()
            .matches(&m)
    );
}

#[test]
fn range_filter_large_unsigned_vs_float_precision_regression() {
    let mut m = Metadata::new();
    m.set("n", 9_007_199_254_740_993_u64);

    assert!(
        MetadataFilter::builder()
            .gt("n", 9_007_199_254_740_992_f64)
            .build()
            .matches(&m)
    );
    assert!(
        MetadataFilter::builder()
            .ge("n", 9_007_199_254_740_992_f64)
            .build()
            .matches(&m)
    );
}

#[test]
fn range_filter_float_vs_integer_and_huge_unsigned() {
    let huge_u = (i64::MAX as u64) + 1;

    let mut m = Metadata::new();
    m.set("ratio", 3.5_f64);
    assert!(
        MetadataFilter::builder()
            .gt("ratio", 3_i64)
            .build()
            .matches(&m)
    );

    let mut n = Metadata::new();
    n.set("value", 9_223_372_036_854_777_856_f64);
    assert!(
        MetadataFilter::builder()
            .gt("value", huge_u)
            .build()
            .matches(&n)
    );
}

#[test]
fn range_filter_large_integer_float_non_integral_fallback() {
    let mut signed = Metadata::new();
    signed.set("n", 9_007_199_254_740_993_i64);
    assert!(
        !MetadataFilter::builder()
            .gt("n", 0.5_f64)
            .build()
            .matches(&signed)
    );

    let mut unsigned = Metadata::new();
    unsigned.set("n", (i64::MAX as u64) + 123);
    assert!(
        !MetadataFilter::builder()
            .gt("n", 0.5_f64)
            .build()
            .matches(&unsigned)
    );
    assert!(
        MetadataFilter::builder()
            .gt("n", -1.0_f64)
            .build()
            .matches(&unsigned)
    );
}

#[test]
fn approximate_number_policy_enables_lossy_fallback_for_large_i64() {
    let mut m = Metadata::new();
    m.set("n", 9_007_199_254_740_993_i64);

    let conservative = MetadataFilter::builder().gt("n", 0.5_f64);
    assert!(!conservative.clone().build().matches(&m));

    let approximate = conservative
        .clone()
        .number_comparison_policy(NumberComparisonPolicy::Approximate);
    assert!(approximate.build().matches(&m));
}

#[test]
fn approximate_number_policy_enables_lossy_fallback_for_large_u64() {
    let mut m = Metadata::new();
    m.set("n", (i64::MAX as u64) + 123);

    let conservative = MetadataFilter::builder().gt("n", 0.5_f64);
    assert!(!conservative.clone().build().matches(&m));

    let approximate = conservative
        .clone()
        .number_comparison_policy(NumberComparisonPolicy::Approximate);
    assert!(approximate.build().matches(&m));
}

#[test]
fn range_filter_incomparable_types_do_not_match() {
    assert!(
        !MetadataFilter::builder()
            .gt("status", 1_i64)
            .build()
            .matches(&sample())
    );
    assert!(
        !MetadataFilter::builder()
            .lt("verified", 1_i64)
            .build()
            .matches(&sample())
    );
}

#[test]
fn exists_present_key() {
    assert!(
        MetadataFilter::builder()
            .exists("status")
            .build()
            .matches(&sample())
    );
}

#[test]
fn exists_missing_key() {
    assert!(
        !MetadataFilter::builder()
            .exists("nope")
            .build()
            .matches(&sample())
    );
}

#[test]
fn not_exists_missing_key() {
    assert!(
        MetadataFilter::builder()
            .not_exists("nope")
            .build()
            .matches(&sample())
    );
}

#[test]
fn not_exists_present_key() {
    assert!(
        !MetadataFilter::builder()
            .not_exists("status")
            .build()
            .matches(&sample())
    );
}

#[test]
fn in_values_matches() {
    let f = MetadataFilter::builder().in_set("status", ["active", "pending"]);
    assert!(f.build().matches(&sample()));
}

#[test]
fn in_values_no_match() {
    let f = MetadataFilter::builder().in_set("status", ["inactive", "pending"]);
    assert!(!f.build().matches(&sample()));
}

#[test]
fn in_values_missing_key() {
    let f = MetadataFilter::builder().in_set("missing", ["x"]);
    assert!(!f.build().matches(&sample()));
}

#[test]
fn not_in_values_matches() {
    let f = MetadataFilter::builder().not_in_set("status", ["inactive", "pending"]);
    assert!(f.build().matches(&sample()));
}

#[test]
fn not_in_values_no_match() {
    let f = MetadataFilter::builder().not_in_set("status", ["active", "pending"]);
    assert!(!f.build().matches(&sample()));
}

#[test]
fn not_in_values_missing_key_matches() {
    let f = MetadataFilter::builder().not_in_set("missing", ["x"]);
    assert!(f.build().matches(&sample()));
}

#[test]
fn not_in_values_missing_key_respects_policy() {
    let f = MetadataFilter::builder().not_in_set("missing", ["x"]);
    let strict = f.clone().missing_key_policy(MissingKeyPolicy::NoMatch);
    assert!(f.build().matches(&sample()));
    assert!(!strict.build().matches(&sample()));
}

#[test]
fn missing_key_policy_applies_recursively_in_filter_tree() {
    let f = MetadataFilter::builder()
        .ne("missing", "x")
        .and_not_in_set("missing-2", ["y"])
        .or_eq("status", "inactive");
    assert!(f.clone().build().matches(&sample()));

    let strict = f.missing_key_policy(MissingKeyPolicy::NoMatch);
    assert!(!strict.build().matches(&sample()));
}

#[test]
fn condition_serde_round_trip() {
    let c = Condition::Equal {
        key: "status".into(),
        value: qubit_value::Value::from_json_value(serde_json::json!("active")),
    };
    let json = serde_json::to_string(&c).unwrap();
    let restored: Condition = serde_json::from_str(&json).unwrap();
    assert_eq!(c, restored);
}
