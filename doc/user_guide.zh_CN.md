# qubit-metadata 用户手册

[English User Guide](user_guide.md) · [从 0.6 升级到 0.7](migration_0.7.zh_CN.md) · [迁移到 0.6](migration_0.6.zh_CN.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-metadata)

本手册面向使用 `qubit-metadata` 0.7 和 Rust 1.94 及以上版本的 Rust 开发者。它适合
需要给记录、消息或文档分片附加可类型化、可查询 metadata，同时又不希望把 metadata
模型绑定到某个存储 provider 的场景。

## 问题与模型

`Metadata` 是从字符串 key 到 `qubit_value::Value` 的有序映射。它可以扩展，但每个值仍
保留具体类型。因此，调用方可以在允许缺失时使用方便的类型化读取，也可以在必须区分
键缺失和转换失败时使用带诊断的读取。

可选能力建立在核心容器之上：

```text
Metadata -> MetadataSchema -> 已存值校验
        -> FilterExpression -> MetadataFilter -> 匹配
        -> 严格的 V1 Serde/JSON wire format
```

`schema` 会启用 schema 校验并包含 `filter`。如果不需要 schema 校验，可以单独启用
`filter`。`json` feature 提供带资源限制的 JSON slice 解码。

## 场景：索引文档分片

假设一条流水线接收文档、切分分片，再把每个分片发送给存储后端。一个成功的实现应能够：

1. 保留标量类型并附加来源、租户等字段；
2. 在写入存储前拒绝不完整记录；
3. 构造可以翻译或本地执行的 filter；
4. 诊断非法 metadata 或不可信序列化输入。

下面按步骤完成这条路径。

## 安装与 feature 选择

使用核心 metadata API（crate 默认只启用核心功能）：

```toml
[dependencies]
qubit-metadata = "0.7"
```

使用可选能力时请显式启用对应 feature：

<!-- installation-dependencies:start -->
```toml
[dependencies]
qubit-metadata = { version = "0.7", features = ["schema", "json"] }
qubit-datatype = "0.15"
qubit-value = "0.13"
```
<!-- installation-dependencies:end -->

后文构造 `Value::Unset` 的示例直接导入 `qubit_value::Value`，因此也需要
上面列出的 `qubit-value` 依赖。

`schema` 会包含 `filter`；不需要 schema 校验时可使用
`features = ["filter"]`。crate 没有默认 feature，因此只依赖 metadata 容器时
无需额外配置。

crate 声明了 `filter`、`schema`、`chrono`、`big-integer`、`big-decimal`、`big-number`、
`url`、`json` 和 `all`。`all` 会启用声明的可选值类型族和 JSON 支持。

如果希望显式记录“只使用 metadata”的意图，可以关闭默认 feature：

```toml
[dependencies]
qubit-metadata = { version = "0.7", default-features = false }
```

这与当前核心功能默认配置等价，同时能明确记录应用自身的 feature 边界。

## 核心工作流

### 1. 存储有类型字段

`with` 会消费并返回 metadata，适合在一个构造表达式中使用。`set` 修改已有值并返回
可继续链式调用的可变引用。替换 key 时，`insert` 返回旧值。

```rust
use qubit_metadata::Metadata;

let mut metadata = Metadata::new()
    .with("tenant_id", "acme")
    .with("document_id", "doc-42")
    .with("chunk_index", 3_i64)
    .with("language", "en");

metadata.set("indexed", true);
let previous = metadata.insert("chunk_index", 4_i64);
assert!(previous.is_some());
```

key 会按顺序参与迭代和序列化。`merge` 与 `merged` 遇到相同 key 时都使用右侧 metadata
中的值。

### 1a. 保持跨组件 key 稳定

Metadata key 是普通字符串，拼写差异会静默地产生不同字段。应在负责该 key 的边界定义
字符串常量，并在所有写入、读取和 filter 中复用：

```rust
use qubit_metadata::Metadata;

const TENANT_ID: &str = "tenant_id";

let metadata = Metadata::new().with(TENANT_ID, "acme");
assert_eq!(metadata.get_ref::<str>(TENANT_ID).unwrap(), "acme");
```

如果 key/value 契约需要和存储 provider 共享，应增加 `MetadataSchema`，并在跨越边界前校验
metadata。本 crate 不会替调用方规范化 key 的拼写或命名风格。

### 2. 选择合适的读取失败模型

`get` 严格读取并返回 `Result<T>`，`get_ref` 借用类型化载荷，`get_optional` 返回
`Result<Option<T>>` 且保留类型错误。检查原始 `Value` 时使用 `get_raw`，需要转换时使用
`convert`。与 `Config::get` 不同，`Metadata::get` 不执行隐式转换。

```rust
use qubit_metadata::{Metadata, MetadataError};

let metadata = Metadata::new().with("chunk_index", 3_i64);

let index: Option<i64> = metadata.get_optional("chunk_index").unwrap();
assert_eq!(index, Some(3));

match metadata.get::<String>("chunk_index") {
    Err(MetadataError::ValueAccess { .. }) => {}
    other => panic!("unexpected result: {other:?}"),
}
```

`Value::Unset` 与 key 缺失不同：它仍然存在并保留声明类型，但不包含具体值。key 缺失时，
`get` 返回 `MissingKey`。类型符合要求的 unset 返回包含 `ValueError::Missing` 的
`ValueAccess`；声明类型不匹配仍然是类型错误。可选/默认值读取只吸收允许回退的缺失状态。

### 3. 校验并索引分片

下面的 schema 和查询使用同一组分片字段：`tenant_id`、`chunk_index` 和可选的
`language`。调用应用的存储接口前，先校验每条记录。使用
`build_checked(&schema)` 会另外检查查询字段和操作数类型，再执行查询。这里的本地校验
不能替代存储层的授权或租户隔离。

<!-- chunk-workflow-example:start -->
```rust
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
```
<!-- chunk-workflow-example:end -->

合法记录会被接收；缺少 `chunk_index` 的记录会在调用 `insert` 前被拒绝，因此存储计数仍为
1。已检查的查询匹配同一租户，不匹配其他租户。若整数范围条件使用字符串操作数，查询会在
`build_checked` 阶段被拒绝。`RecordingStore` 只是让这些路径可运行的测试替身。应用应使用
自己的持久化实现接入 `ChunkStore`，并负责事务、重试和查询翻译。

### 4. 为另一个查询组合谓词

未指定分组连接词的谓词按 AND 连接。需要嵌套表达式时使用 `and_group` 和 `or_group`：

```rust
use qubit_metadata::{FilterExpression, Metadata, MetadataFilter};

let expression = FilterExpression::builder()
    .eq("status", "ready")
    .and_group(|group| {
        group
            .ge("score", 80_i64)
            .or_group(|alternative| alternative.eq("tag", "rust"))
    })
    .build()
    .unwrap();
let filter = MetadataFilter::builder()
    .expression(expression)
    .build()
    .unwrap();

let metadata = Metadata::new()
    .with("status", "ready")
    .with("score", 42_i64)
    .with("tag", "rust");
assert!(filter.matches(&metadata));
```

该表达式等价于 `status == "ready" AND (score >= 80 OR tag == "rust")`。其他 builder
谓词包括 `ne`、`gt`、`lt`、`le`、`exists`、`not_exists`、`in_set`、`not_in_set` 和 `not`。

## 从旧版 API 迁移

若从 0.6 之前的版本升级，或仍有依赖隐式转换、返回 `Option` 的 `get`、
`MetadataError::MissingValue` 的调用点，请在完成上述核心工作流后继续阅读
[迁移到 0.6：明确读取方式与边界诊断](migration_0.6.zh_CN.md)。新项目可跳过该文档，
直接使用下文各节。

## 进阶用法

### Filter 的三值逻辑

对缺失 key 或 `Value::Unset` 进行相等、范围和集合比较会得到 unknown。具体值的类型或运算不兼容、
数值比较失败时也可能得到 unknown。不同类型的值不会隐式转换；兼容数值类型由
`NumericComparisonPolicy` 决定。取反会保留 unknown，因此 `ne`、否定比较和 `not_in_set` 不能绕过
不可比较的类型。集合条件逐项比较候选，并由确定结果优先于 unknown 候选。公开的 `matches` 只有在结果
明确为 true 时才返回 `true`。存在性谓词仍返回确定值：没有存储具体值时，`exists` 为 false、
`not_exists` 为 true；所以 `exists("key").not()` 也为 true。布尔组合遵循确定值优先：
`false AND unknown` 为 `false`，`true OR unknown` 为 `true`，其余混合情况保持 unknown。

```rust
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;

let metadata = Metadata::new().with("age", "not-a-number");
let expressions = [
    FilterExpression::builder().ne("age", 18_i64).build().unwrap(),
    FilterExpression::builder().eq("age", 18_i64).not().build().unwrap(),
    FilterExpression::builder().not_in_set("age", [18_i64, 21_i64]).build().unwrap(),
];
for expression in expressions {
    let filter = MetadataFilter::builder().expression(expression).build().unwrap();
    assert!(!filter.matches(&metadata));
}
```

空集合是合法的。请为 empty iterator 指定元素类型，因为空数组无法为 `in_set` 或
`not_in_set` 推断值类型。此示例需要启用 `filter` feature：

<!-- empty-set-example:start -->
```rust
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
use qubit_value::Value;

pub fn main() -> qubit_metadata::MetadataResult<()> {
    let included = MetadataFilter::builder()
        .expression(
            FilterExpression::builder()
                .in_set("key", std::iter::empty::<i64>())
                .build()?,
        )
        .build()?;
    let excluded = MetadataFilter::builder()
        .expression(
            FilterExpression::builder()
                .not_in_set("key", std::iter::empty::<i64>())
                .build()?,
        )
        .build()?;
    let concrete = Metadata::new().with("key", 1_i64);
    assert!(!included.matches(&concrete));
    assert!(excluded.matches(&concrete));
    assert!(!excluded.matches(&Metadata::new()));
    assert!(!excluded.matches(&Metadata::new().with("key", Value::Unset(DataType::Int64))));
    Ok(())
}
```
<!-- empty-set-example:end -->

空分组会被 `build()` 拒绝，因为它通常说明查询构造过程遗漏了条件。

数值比较使用配置的 `NumericComparisonPolicy`。filter 的 schema 校验会把非 NaN 数值表示
视为与数值字段兼容；已存 metadata 的 schema 校验仍严格要求声明的具体类型。近似投影比较
依赖操作数对且不满足传递性，因此不能用于排序、分组、`Ord` 或有序 key。

### 检查或翻译 expression

`MetadataFilter::expression()` 返回根 expression，`FilterExpression::view()` 提供只读树，包含
condition、AND、OR、NOT、true 和 false 节点。存储 provider 可以遍历该 view，将支持的谓词翻译为
原生查询语言，而不依赖私有构造表示。表达式树只描述结构，本身不保证后端翻译保持语义；数值行为
也属于 filter 选项，可通过 `filter.options().numeric_comparison_policy()` 获取。

翻译必须让三值结果完整经过 NOT、AND、OR 运算，最后只匹配结果为 `true` 的记录。尤其不能在
应用 NOT 前先把 unknown 比较转换为 false。匹配规则见
[Filter 的三值逻辑](#filter-的三值逻辑)。`exists` 检查的是 concrete value，而不是 map 是否包含该 key；
unset 值不算存在。membership 必须保留不可比较候选、确定匹配优先规则及空集合行为。数值比较也必须
遵循已配置的策略，不能默认采用后端的强制转换规则。

下面的示例说明，将 `NOT (tenant = "acme")` 翻译成普通不等比较，可能会错误地纳入缺少 tenant 值的
记录。对于下面的空 metadata，NOT 包裹的比较不匹配，而 `not_exists` 匹配。如果某个谓词无法按等价
语义翻译，应拒绝该谓词；或者仅在能够取得完整候选集后进行本地匹配。不能先执行会漏掉匹配项的下推、
分页、limit 或聚合，再把本地过滤结果视为等价。应按应用需求在语义过滤完成后排序和分页。

<!-- query-translation-example:start -->
```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;

pub fn main() -> qubit_metadata::MetadataResult<()> {
    let metadata = Metadata::new();
    let comparison = MetadataFilter::builder()
        .expression(FilterExpression::builder().eq("tenant", "acme").not().build()?)
        .build()?;
    let absent = MetadataFilter::builder()
        .expression(FilterExpression::builder().not_exists("tenant").build()?)
        .build()?;
    assert!(!comparison.matches(&metadata));
    assert!(absent.matches(&metadata));
    Ok(())
}
```
<!-- query-translation-example:end -->

### 有界 JSON 解码

启用 `json` 后，对于完整的不可信 JSON 输入使用 slice 解码方法：

<!-- bounded-json-decoding-example:start -->
```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;
use qubit_metadata::MetadataWireDecodeError;

pub fn main() -> Result<(), MetadataWireDecodeError> {
    let limits = MetadataLimits::builder()
        .max_metadata_entries(128)
        .max_key_bytes(128)
        .build()
        .map_err(MetadataWireDecodeError::InvalidLimits)?;
    let metadata = Metadata::decode_json_slice_with_limits(
        br#"{"version":1,"values":{"tenant_id":{"scalar":{"string":"acme"}}}}"#,
        limits,
    )?;
    assert_eq!(
        metadata.get_ref::<str>("tenant_id").expect("tenant_id is a string"),
        "acme"
    );
    Ok(())
}
```
<!-- bounded-json-decoding-example:end -->

默认限制是输入 1,048,576 字节、4,096 个 metadata 条目、4,096 个 schema 字段，以及每个
key 256 个 UTF-8 字节。`MetadataLimits` 将这些 metadata 领域限制与内部的
`JsonDecodeLimits`、`JsonEncodeLimits` profile 分开；使用 `MetadataLimitsBuilder` 的
`json_decode` 或 `json_encode` 方法替换相应的通用遍历和定向字节限制。领域限制不能超过 V1 序列化的规范
硬上限。输入字节数会在 JSON 解析前检查。

`MetadataSchema` 和 `MetadataFilter` 也提供对应的有界 JSON 解码器。Filter 解码还接受由
接收方控制的 `FilterLimits`。显式 filter JSON decoder 首先在共享 JSON 解码预算下物化完整的
唯一键 JSON 源树，然后由领域 seed 在构造 filter 时检查接收方 AST 和 membership 限制。收紧
`FilterLimits` 不会停止前一阶段的源树物化，因此应同时配置通用 JSON budget 和领域限制。
Filter limits 是接收端瞬态策略，不会被序列化。单个 JSON
字符串和嵌套 value 仍可能产生临时分配，但会受到外层输入字节上限约束。通用
`serde::Deserialize` 适用于外层已经受控的协议。

如果 limits 来自运维配置或配置文件，应使用
`MetadataLimits::builder().build()?`，在配置阶段拒绝超过协议硬上限的领域限制，
与 `FilterLimitsBuilder::build` 一致。

### 定向 JSON 预算

解码和编码预算相互独立。此例将输入上限设为 64 KiB、输出上限设为
128 KiB。Metadata 领域限制描述 V1 metadata 契约；JSON 限制约束通用 JSON
遍历和字节数。

<!-- directional-json-limits-example:start -->
```rust
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;
use qubit_metadata::default_json_decode_limits;
use qubit_metadata::default_json_encode_limits;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let decode = default_json_decode_limits()
        .into_builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 64 * 1024))
        .build();
    let encode = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 128 * 1024))
        .build();
    let limits = MetadataLimits::builder()
        .json_decode(decode)
        .json_encode(encode)
        .build()?;
    assert_eq!(limits.json_decode().max_input_bytes(), Some(64 * 1024));
    assert_eq!(limits.json_encode().max_output_bytes(), Some(128 * 1024));

    let metadata = Metadata::new().with("tenant_id", "acme");
    let bytes = metadata.to_json_vec_with_limits(limits.json_encode())?;
    assert!(!bytes.is_empty());
    Ok(())
}
```
<!-- directional-json-limits-example:end -->

### 有界 JSON 编码

文档分片准备写入存储或发送到后端时，应在应用自己的边界完成编码，再把生成的字节交给
持久化或传输代码。下面的完整最小程序设置输出预算、编码 metadata、重新解码并检查结果，
同时验证 writer 输出与 vec 编码一致：

```toml
[dependencies]
qubit-metadata = { version = "0.7", features = ["json"] }
qubit-budget = { version = "0.10", features = ["json"] }
```

<!-- bounded-json-encoding-example:start -->
```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;
use qubit_metadata::default_json_encode_limits;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let encode = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 128 * 1024))
        .build();
    let limits = MetadataLimits::builder().json_encode(encode).build()?;
    let metadata = Metadata::new()
        .with("tenant_id", "acme")
        .with("document_id", "doc-42")
        .with("chunk_index", 3_i64);

    let bytes = metadata.to_json_vec_with_limits(limits.json_encode())?;
    // The application passes these bytes to its own storage or transport layer.
    let restored = Metadata::decode_json_slice_with_limits(&bytes, limits)?;
    assert_eq!(restored, metadata);

    let mut output = Vec::new();
    metadata.to_json_writer_with_limits(&mut output, limits.json_encode())?;
    assert_eq!(output, bytes);
    Ok(())
}
```
<!-- bounded-json-encoding-example:end -->

这只是帮助熟悉 API 的入门练习，并未实现存储。真实应用仍需负责事务、发送、重试，以及
I/O 失败后的补偿。`to_json_vec` 和 `to_json_writer` 使用默认输出预算；若
`MetadataLimits` 配置了自定义编码 profile，调用时要显式传入 `limits.json_encode()`。
修改 `json_decode` 不会改变输出 profile。启用 `schema` feature 后，`MetadataSchema`
也提供同一组 `to_json_vec*` 和 `to_json_writer*` 方法。 Metadata 和 schema 的有界编码遇到 V1 wire 限额时都会返回 `Domain(MetadataError::WireLimitExceeded { kind, value, maximum })`，并优先于 JSON 输出预算检查。原先针对 schema 限额匹配 `Json` 的调用方应改为匹配 `Domain` 并读取这些字段；普通 Serde 序列化错误仍为 `Json`。

编码器会先检查值是否可表示以及预算是否足够，再生成完整的紧凑 JSON 缓冲区；writer API
随后写出缓冲结果。若目标 writer 写入失败，目标端仍可能已经收到部分字节，因此不能据此
认为文件或网络写入具有事务原子性。将输出预算设为 1 字节可以稳定观察拒绝结果：

```rust
use qubit_budget::json::JsonResource;
use qubit_budget::ResourceLimit;
use qubit_metadata::default_json_encode_limits;
use qubit_metadata::{Metadata, MetadataWireEncodeError};

let limits = default_json_encode_limits()
    .into_builder()
    .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
    .build();
let metadata = Metadata::new().with("tenant_id", "acme");
assert!(matches!(
    metadata.to_json_vec_with_limits(limits),
    Err(MetadataWireEncodeError::Budget(_))
));
```

遇到预算错误时，应缩减输出，或按应用策略调整预算；不要对同一载荷改用无限额配置重试。
`MetadataWireEncodeError` 还会区分 `Quantity`、`Syntax`、`Json` 和 `Io`。Metadata 与 schema
wire 契约错误使用 `Domain(MetadataError::WireLimitExceeded { kind, value, maximum })`，保留限制类别和实际测量值。schema 编码会先执行领域限制 preflight，再检查 JSON 输出预算。迁移原先将 schema 限额错误视为 `Json` 的代码时，应改为匹配 `Domain` 并读取这些字段；普通 Serde 序列化错误仍为 `Json`。该枚举标记为 `non_exhaustive`，穷举匹配时应保留兜底分支。

启用 `json` 和 `filter` 后，`MetadataFilter` 也支持有界 JSON 编码。`to_json_vec` 和
`to_json_writer` 使用默认 JSON 输出预算；`to_json_vec_with_limits` 和
`to_json_writer_with_limits` 接收显式的 `JsonEncodeLimits`。这些限制约束编码输出。
接收端的 `FilterLimits` 是瞬态策略，不会序列化。编码器会在 writer 写入首字节前完成文档
编码和预算检查，因此预算失败不会写入目标；目标 I/O 失败仍可能留下部分前缀。

```toml
[dependencies]
qubit-metadata = { version = "0.7", features = ["json", "filter"] }
```

下面的示例构造 tenant filter，编码并解码后检查匹配结果，同时验证 writer 和 Vec API
生成相同的字节：

<!-- bounded-filter-json-encoding-example:start -->
```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
use qubit_metadata::default_json_encode_limits;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let expression = FilterExpression::builder().eq("tenant", "acme").build()?;
    let filter = MetadataFilter::builder().expression(expression).build()?;
    let bytes = filter.to_json_vec_with_limits(default_json_encode_limits())?;
    let restored = MetadataFilter::decode_json_slice(&bytes)?;
    assert!(restored.matches(&Metadata::new().with("tenant", "acme")));
    let mut output = Vec::new();
    filter.to_json_writer_with_limits(&mut output, default_json_encode_limits())?;
    assert_eq!(output, bytes);
    Ok(())
}
```
<!-- bounded-filter-json-encoding-example:end -->

### 严格的 V1 wire format

Metadata、schema 和 filter 都通过严格的 V1 envelope 序列化。未知字段、畸形节点和不支持
的版本都会被拒绝。Filter envelope 只包含 `version`、`expression` 和 `options`；旧 V1
载荷中的 `limits` 会作为未知字段被拒绝。expression 节点使用 `eq`、`ge`、`in`、`and`、
`or`、`not`、`all` 和 `none` 等 tag。
除非版本化格式本身是集成契约，否则不要手写这些结构，优先使用公开的 Serde 实现。

schema 的 `fields` map 中每个值必须是仅含 `data_type` 和 `required` 的对象；位置式字段数组会被
拒绝。若已存 V1 schema 使用字段数组，升级前应使用旧版本读取，再以旧版本重编码为规范对象；
新 decoder 不提供数组回退。

`Metadata` 和 `MetadataSchema` 在内存模型中不限制条目数和 key 长度。其 V1 wire 契约将
map 条目数限制为 4,096，key 长度限制为 256 个 UTF-8 字节。通用 Serde 序列化返回
serializer 的错误类型（`S::Error`），例如 `serde_json::Error`；领域错误会通过 `custom`
转换，因此调用方不能直接在该返回值上匹配 `MetadataWireEncodeError::Domain`。有界的
`to_json_vec*` 和 `to_json_writer*` helper 则返回 `MetadataWireEncodeError`，并将 wire 限额错误
保留为 `Domain(MetadataError::WireLimitExceeded { .. })`，且先检查领域限制，再检查 JSON
输出预算。若需要结构化的 `MetadataError` 详情，可在通用序列化前调用
`validate_wire_contract()`。这些硬性 wire 限制与默认严格解码器共享；生成交换数据时应在生产端
保持在限制以内。针对特定边界时，接收方控制的 JSON 限制可以进一步降低资源上限。

## 错误与诊断

根据校验边界选择错误类型：

| 边界 | API | 典型信息 |
| --- | --- | --- |
| 单次 metadata 读取 | `get` / `convert` | key 缺失、unset、转换/类型不匹配 |
| 单次 schema 校验 | `MetadataSchema::validate` | 通过 `issues()` 获取所有独立 metadata 问题 |
| Filter 构造 | `build` / `build_checked` | 空分组、非法操作数、未知字段、操作符不兼容 |
| JSON 输入 | `decode_json_slice_with_limits` | JSON budget、JSON 非法或 V1 校验失败 |

转换诊断会保留期望类型和实际类型等结构化信息，而不会嵌入被拒绝的原始值。这使错误
适合写入日志，同时减少意外泄露 metadata 内容的风险。

有界 JSON 解码仍将 `Read`、`Budget`、`Quantity` 和 `Syntax` 分开报告。共享 decoder
产生的空输入、非法 UTF-8、顶层类型不符或反序列化失败则归为
`MetadataWireDecodeError::Decode`；应通过 `kind()` 等 accessor 读取结构化信息，不要解析
展示文本。标准 metadata 入口使用 `Redacted` 诊断策略。只有调用方在转换前明确选择
`Detailed` 时，详细来源才会保留。仍可手工构造旧的 `InvalidJson`；如果原有匹配代码假设它
涵盖所有 JSON 失败，应增加 `Decode` 分支。错误字符串和来源链深度不属于兼容性保证。

## 排障

### Filter 意外没有匹配

先判断谓词类别。对缺失或 unset 值进行相等、范围和集合比较会得到 unknown，因此不会匹配；
存在性谓词则返回确定结果（没有存储具体值时，`exists` 为 false、`not_exists` 为 true）。
再检查声明的 `DataType` 和数值比较策略。对于嵌套 filter，可以检查 `expression().view()`
确认分组和连接词结构。

### Schema 拒绝了可以进行数值比较的值

Filter 兼容性检查和已存 metadata 校验的目的不同。前者为构造查询而接受兼容的数值表示；
后者要求已存值使用字段声明的具体类型。存储前转换值，或按实际值类型声明 schema。

### JSON 解码在看起来开始解析前就失败

将输入字节数与 `MetadataLimits` 内部 `JsonDecodeLimits` profile 的输入上限比较。输入大小检查会
有意在 JSON 解析器调用前执行。如果输入在限制内，再根据返回的 `MetadataWireDecodeError`
检查领域限制和通用 JSON 预算事实。

### getter 不再返回 `Option`

0.6 的 `get` 改为可能失败的严格读取。允许缺失时使用 `get_optional` 并传播 `Result`；
需要按策略转换时使用 `convert_optional_with`。非法值返回错误，不再折叠为 `None`。
完整 API 对照见 [0.6 迁移说明](migration_0.6.zh_CN.md)。

## 限制与最佳实践

- 保持 metadata key 和 value 足够小，以适应接收方存储系统；本 crate 不提供存储 provider
  的索引策略。
- 在 metadata 进入可信领域边界时执行 schema 校验，并在解析不可信 JSON 前应用显式 wire
  限制。
- 有 schema 时使用 `MetadataFilterBuilder::build_checked`；只有在目标后端负责字段校验时才使用
  unchecked builder。
- 将诊断格式化视为有界、脱敏的输出，不要把它当成任意用户 key 或错误消息的完整安全策略。
- 序列化 metadata 和 schema map 时遵守 4,096 个条目及 256 字节 key 的 wire 限制。
- 将 V1 Serde 表示放在集成边界后面，以便未来 wire version 变化时能够有意识地处理。

## 下一步

- 阅读 [README](../README.zh_CN.md) 了解项目概览和安装摘要。
- 浏览 [API 文档](https://docs.rs/qubit-metadata) 查看完整公共 API。
- 升级旧调用点时阅读[迁移到 0.6](migration_0.6.zh_CN.md)。
- 阅读[设计文档](design.zh_CN.md)了解模块边界和兼容性不变量。
- 修改 feature-gated 行为前运行 `cargo test --all-features`。
- 查看[英文用户手册](user_guide.md)获取相同流程的英文版本。
