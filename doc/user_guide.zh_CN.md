# qubit-metadata 用户手册

[English User Guide](user_guide.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-metadata)

本手册面向使用 `qubit-metadata` 0.10 和 Rust 1.94 及以上版本的 Rust 开发者。它适合
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
qubit-metadata = "0.10"
qubit-datatype = "0.10"
```

使用可选能力时请显式启用对应 feature：

```toml
[dependencies]
qubit-metadata = { version = "0.10", features = ["schema", "json"] }
qubit-datatype = "0.10"
```

`schema` 会包含 `filter`；不需要 schema 校验时可使用
`features = ["filter"]`。crate 没有默认 feature，因此只依赖 metadata 容器时
无需额外配置。

crate 声明了 `filter`、`schema`、`chrono`、`big-integer`、`big-decimal`、`big-number`、
`url`、`json` 和 `all`。`all` 会启用声明的可选值类型族和 JSON 支持。

如果希望显式记录“只使用 metadata”的意图，可以关闭默认 feature：

```toml
[dependencies]
qubit-metadata = { version = "0.10", default-features = false }
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
assert_eq!(metadata.get_str(TENANT_ID), Some("acme"));
```

如果 key/value 契约需要和存储 provider 共享，应增加 `MetadataSchema`，并在跨越边界前校验
metadata。本 crate 不会替调用方规范化 key 的拼写或命名风格。

### 2. 选择合适的读取失败模型

当键缺失和转换失败都应视为不存在时使用 `get`。需要不经转换检查存储的 `Value` 时使用
`get_raw`。当诊断信息属于应用行为时使用 `try_get`。

```rust
use qubit_metadata::{Metadata, MetadataError};

let metadata = Metadata::new().with("chunk_index", 3_i64);

let index: Option<i64> = metadata.get("chunk_index");
assert_eq!(index, Some(3));

match metadata.try_get::<String>("chunk_index") {
    Err(MetadataError::TypeMismatch { .. }) => {}
    other => panic!("unexpected result: {other:?}"),
}
```

`Value::Unset` 与 key 缺失不同：它仍然存在并保留声明类型，但不包含具体值。key 缺失时，
`try_get` 返回 `MissingKey`；unset 值则返回 `MissingValue`。

### 3. 在存储边界定义 schema

当后端要求固定字段布局，或记录必须在写入前校验时，使用
`MetadataSchema::builder()`。required 字段必须存在且有具体值；optional 字段可以省略。
已存值必须匹配字段声明的具体 `DataType`。

```rust
use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataSchema};

let schema = MetadataSchema::builder()
    .required("tenant_id", DataType::String)
    .required("chunk_index", DataType::Int64)
    .optional("language", DataType::String)
    .build()
    .expect("schema should be valid");

let metadata = Metadata::new()
    .with("tenant_id", "acme")
    .with("chunk_index", 3_i64)
    .with("language", "en");

schema.validate(&metadata).expect("metadata should match schema");
```

未知 metadata 字段和未知 filter 字段使用相互独立的策略。默认策略拒绝未知名称；允许未
声明的 metadata key，不会自动允许未经检查的 filter key。

### 4. 构造并执行 filter

`FilterExpression::builder()` 描述布尔结构。构造出的 expression 再由
`MetadataFilter::builder()` 绑定匹配选项和资源限制。如果希望在 filter 发送到后端前
校验字段和操作数类型，使用 `build_checked(&schema)`。

```rust
use qubit_datatype::DataType;
use qubit_metadata::{
    FilterExpression,
    Metadata,
    MetadataFilter,
    MetadataSchema,
};

let schema = MetadataSchema::builder()
    .required("status", DataType::String)
    .required("score", DataType::Int64)
    .build()
    .unwrap();

let expression = FilterExpression::builder()
    .eq("status", "ready")
    .ge("score", 80_i64)
    .build()
    .unwrap();
let filter = MetadataFilter::builder()
    .expression(expression)
    .build_checked(&schema)
    .unwrap();

let metadata = Metadata::new()
    .with("status", "ready")
    .with("score", 92_i64);
assert!(filter.matches(&metadata));
```

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

## 进阶用法

### Filter 的三值逻辑

缺失 key 和 `Value::Unset` 的结果是 unknown。公开的 `matches` 只有在结果明确为 true 时
才返回 `true`。取反会保留 unknown，AND/OR 会传播 unknown。因此，`ne("key", value)`
和 `not(eq("key", value))` 都不会匹配缺失或 unset key。

空集合是合法的：`in_set("key", [])` 什么都不匹配；`not_in_set("key", [])` 只匹配具体值。
空分组会被 `build()` 拒绝，因为它通常说明查询构造过程遗漏了条件。

数值比较使用配置的 `NumericComparisonPolicy`。filter 的 schema 校验会把非 NaN 数值表示
视为与数值字段兼容；已存 metadata 的 schema 校验仍严格要求声明的具体类型。近似投影比较
依赖操作数对且不满足传递性，因此不能用于排序、分组、`Ord` 或有序 key。

### 检查或翻译 expression

`MetadataFilter::expression()` 返回根 expression，`FilterExpression::view()` 暴露只读树，
其中包括 condition、AND、OR、NOT、true 和 false 节点。存储 provider 可以遍历该 view，
把支持的谓词翻译为自己的查询语言，而不依赖私有构造表示。

### 有界 JSON 解码

启用 `json` 后，对于完整的不可信 JSON 输入使用 slice 解码方法：

```rust
use qubit_metadata::{Metadata, MetadataWireLimits};

let limits = MetadataWireLimits::default()
    .with_max_metadata_entries(128)
    .with_max_key_bytes(128);
let metadata = Metadata::decode_json_slice_with_limits(
    br#"{"version":1,"values":{"tenant_id":{"scalar":{"string":"acme"}}}}"#,
    limits,
)?;
# Ok::<(), qubit_metadata::MetadataWireDecodeError>(())
```

默认限制是输入 1,048,576 字节、4,096 个 metadata 条目、4,096 个 schema 字段，以及每个
key 256 个 UTF-8 字节。`MetadataWireLimits` 可以降低解码资源限制，metadata 条目数、schema
字段数和 key 长度不能超过 V1 序列化的规范硬上限；`with_wire` 会替换共享的 Value 和 JSON
结构限制。输入字节数会在 JSON 解析前检查。

`MetadataSchema` 和 `MetadataFilter` 也提供对应的有界 JSON 解码器。Filter 解码还接受由
接收方控制的 `FilterLimits`。显式 filter JSON decoder 会在读取 expression tree 时执行接收方
AST 限制和共享 wire budget；sender limits 会在完整 envelope 可用后校验。单个 JSON 字符串和
嵌套 value 仍可能产生临时分配，但会受到外层输入字节上限约束。通用 `serde::Deserialize`
适用于外层已经受控的协议。

### 严格的 V1 wire format

Metadata、schema 和 filter 都通过严格的 V1 envelope 序列化。未知字段、畸形节点和不支持
的版本都会被拒绝。Filter envelope 包含 `version`、`expression`、`options` 和 `limits`；
expression 节点使用 `eq`、`ge`、`in`、`and`、`or`、`not`、`all` 和 `none` 等 tag。
除非版本化格式本身是集成契约，否则不要手写这些结构，优先使用公开的 Serde 实现。

`Metadata` 和 `MetadataSchema` 在内存模型中不限制条目数和 key 长度。序列化器只在 V1
wire 边界拒绝超过 4,096 个条目的 map，或包含超过 256 个 UTF-8 字节 key 的 map，并返回
可读的序列化错误。这些硬性 wire 限制与默认严格解码器共享；生成交换数据时应在生产端保持
在限制以内。针对特定边界时，接收方控制的 JSON 限制可以进一步降低资源上限。

## 错误与诊断

根据校验边界选择错误类型：

| 边界 | API | 典型信息 |
| --- | --- | --- |
| 单次 metadata 读取 | `try_get` | key 缺失、unset、转换/类型不匹配 |
| 单次 schema 校验 | `MetadataSchema::validate` | 通过 `issues()` 获取所有独立 metadata 问题 |
| Filter 构造 | `build` / `build_checked` | 空分组、非法操作数、未知字段、操作符不兼容 |
| JSON 输入 | `decode_json_slice_with_limits` | 输入过大、JSON 非法、wire budget 或 V1 校验失败 |

转换诊断会保留期望类型和实际类型等结构化信息，而不会嵌入被拒绝的原始值。这使错误
适合写入日志，同时减少意外泄露 metadata 内容的风险。

## 排障

### Filter 意外没有匹配

检查 key 是否缺失或存储了 `Value::Unset`；两者都是 unknown，不是 false，因此不会匹配。
检查声明的 `DataType` 和数值比较策略。对于嵌套 filter，可以检查 `expression().view()`
确认分组和连接词结构。

### Schema 拒绝了可以进行数值比较的值

Filter 兼容性检查和已存 metadata 校验的目的不同。前者为构造查询而接受兼容的数值表示；
后者要求已存值使用字段声明的具体类型。存储前转换值，或按实际值类型声明 schema。

### JSON 解码在看起来开始解析前就失败

将输入字节数与 `MetadataWireLimits::max_json_bytes()` 比较。输入大小检查会有意在 JSON
解析器调用前执行。如果输入在限制内，再根据返回的 `MetadataWireDecodeError` 检查条目数、
字段数、key 长度和 Value wire 限制。

### `get` 隐藏了失败原因

这是它的既定契约。调用方需要区分缺失、类型不匹配和 unset 时，应改用 `try_get` 并匹配
`MetadataError`。

## 限制与最佳实践

- 保持 metadata key 和 value 足够小，以适应接收方存储系统；本 crate 不提供存储 provider
  的索引策略。
- 在 metadata 进入可信领域边界时执行 schema 校验，并在解析不可信 JSON 前应用显式 wire
  限制。
- 有 schema 时使用 `MetadataFilter::build_checked`；只有在目标后端负责字段校验时才使用
  unchecked builder。
- 将诊断格式化视为有界、脱敏的输出，不要把它当成任意用户 key 或错误消息的完整安全策略。
- 序列化 metadata 和 schema map 时遵守 4,096 个条目及 256 字节 key 的 wire 限制。
- 将 V1 Serde 表示放在集成边界后面，以便未来 wire version 变化时能够有意识地处理。

## 下一步

- 阅读 [README](../README.zh_CN.md) 了解项目概览和安装摘要。
- 浏览 [API 文档](https://docs.rs/qubit-metadata) 查看完整公共 API。
- 修改 feature-gated 行为前运行 `cargo test --all-features`。
- 查看[英文用户手册](user_guide.md)获取相同流程的英文版本。
