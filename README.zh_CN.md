# Qubit Metadata

[![Rust CI](https://github.com/qubit-ltd/rs-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-metadata.svg?color=blue)](https://crates.io/crates/qubit-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

适用于 Rust 的通用、类型安全元数据模型。

## 概述

`qubit-metadata` 提供 `Metadata` 类型，用于给数据对象附加类型明确的扩展字段，
避免把所有辅助字段都写死进核心结构体。典型场景包括：

- 文档入库：为每个分片保留 `file_id`、`chunk_index`、`language`、`source`、`confidence`。
- 向量检索：保存 `tenant_id`、`doc_type`、`created_at`、`score`、`acl_group` 等后续会进入向量数据库 metadata column 或过滤条件的字段。
- 消息与事件链路：透传 `trace_id`、`request_id`、`tenant_id`、`route`、重试信息等上下文。
- 外部服务集成：记录模型版本、延迟、计费标签、请求编号等便于诊断和统计的紧凑字段。

`Metadata` 底层使用 `qubit_value::Value`，因此标量类型是明确的：`i64` 和
`u32` 不会混成一个模糊的 number，`f64` 和 `String` 也不会混淆。如果确实需要嵌套
结构，可以显式存 `Value::Json`；但常见的文档元信息、向量库 metadata、链路上下文，
通常都是扁平字段集合。
`Value::Unset` 会保留声明类型，但不表示具体 metadata 值：类型化读取返回
`MetadataError::MissingValue`，required schema 字段拒绝它，filter 则将其视为缺失 key。

## 设计目标

- **类型明确**：用 `qubit_value::Value` 保留具体运行时类型。
- **构造方便**：支持可变链式 `set()`、返回旧值的 `insert()` 和取得所有权的
  链式 `with()`。
- **可选 schema**：用 `MetadataSchema` 校验字段名、必填字段和具体 `DataType`。
- **过滤器 builder**：通过 builder 构造不可变的 `MetadataFilter`。
- **序列化友好**：metadata、schema、filter 都支持 `serde`，便于配置、存储和跨服务传输。

## 特性

### 1) 类型化 metadata 容器

`Metadata` 是有序的 `String -> Value` 映射，支持 `get`、链式 `set`、返回旧值的
`insert`、`try_get`、带 schema 校验的 `set_checked` / `insert_checked` /
`with_checked`、链式 `with`、迭代、合并、保留和
`BTreeMap<String, Value>` 转换。

```rust
use qubit_metadata::Metadata;

let meta = Metadata::new()
    .with("author", "alice")
    .with("priority", 3_i64)
    .with("reviewed", true);

assert_eq!(meta.get::<String>("author").as_deref(), Some("alice"));
assert_eq!(meta.try_get::<i64>("priority").unwrap(), 3);
```

可变写入可以链式调用且不会移动 metadata；需要旧值时使用 `insert()`：

```rust
use qubit_metadata::Metadata;

let mut meta = Metadata::new();
meta.set("author", "alice").set("priority", 3_i64);
let previous = meta.insert("priority", 4_i64);
assert!(previous.is_some());
```

`Value::Unset` 记录声明类型，但不提供具体值：

| 操作 | 键缺失 | 存储 `Unset` | 具体值 |
|---|---|---|---|
| `contains_key` | `false` | `true` | `true` |
| `get_raw` | `None` | `Some(Unset)` | `Some(value)` |
| `data_type` | `None` | 声明类型 | 具体类型 |
| `try_get` | `MissingKey` | `MissingValue` | 值或转换错误 |
| required schema 字段 | 拒绝 | 拒绝 | 正常校验 |
| filter `exists` | `false` | `false` | `true` |
| 比较类 filter | 不匹配 | 不匹配 | 正常比较 |

### 2) 用 schema 做校验和存储规划

`MetadataSchema` 使用 `qubit_datatype::DataType`。当存储后端要求预先声明 metadata 字段时，
schema 可以直接作为字段定义来源；在构造 filter 时，也可以提前校验字段、操作符和
过滤值类型是否匹配。`UnknownMetadataFieldPolicy` 和
`UnknownFilterFieldPolicy` 相互独立：允许未声明 metadata 字段并不会同时允许未经校验的
filter 字段。两者默认都拒绝未知字段；只有
`UnknownFilterFieldPolicy::AllowUnchecked` 才会接受未知 filter 字段。

```rust
use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataSchema};

let schema = MetadataSchema::builder()
    .required("tenant_id", DataType::String)
    .required("score", DataType::Int64)
    .optional("source", DataType::String)
    .build()
    .expect("schema should build");

let meta = Metadata::new()
    .with("tenant_id", "acme")
    .with("score", 42_i64);

schema.validate(&meta).unwrap();
```

### 3) builder 构造不可变 filter

`FilterExpression::builder()` 负责构造必需的布尔表达式，
`MetadataFilter::builder()` 则把表达式与匹配选项和资源限制绑定。调用 `build()` 后得到
`Result<MetadataFilter, MetadataError>`。如果已有 schema，可以用 `build_checked(&schema)`
在构建时校验字段是否存在、操作符是否适用于字段类型、过滤值类型是否兼容。schema 级校验会
返回聚合错误，调用方可以一次拿到所有相互独立的问题。

```rust
use qubit_datatype::DataType;
use qubit_metadata::{FilterExpression, Metadata, MetadataFilter, MetadataSchema};

let schema = MetadataSchema::builder()
    .required("status", DataType::String)
    .required("score", DataType::Int64)
    .build()
    .expect("schema should build");

let expression = FilterExpression::builder()
    .eq("status", "active")
    .ge("score", 10)
    .build()
    .unwrap();
let filter = MetadataFilter::builder()
    .expression(expression)
    .build_checked(&schema)
    .unwrap();

let meta = Metadata::new()
    .with("status", "active")
    .with("score", 42_i64);

assert!(filter.matches(&meta));
```

### 4) 过滤 DSL

| 方法 | 含义 |
|------|------|
| `eq` / `ne` | 相等 / 不相等 |
| `gt` / `ge` / `lt` / `le` | 数值范围或字符串字典序比较 |
| `exists` / `not_exists` | 具体值存在 / 不存在 |
| `in_set` / `not_in_set` | 集合包含 / 排除 |
| `and_group` / `or_group` | 追加分组子表达式 |
| `not` | 对当前表达式取反 |

未指定连接词的谓词按 AND 连接。分组子表达式使用闭包构造。闭包会收到一个新的 builder，
闭包返回的表达式会作为一个整体追加到外层表达式中：

```rust
use qubit_metadata::{FilterExpression, Metadata, MetadataFilter};

let expression = FilterExpression::builder()
    .eq("status", "active")
    .and_group(|group| {
        group.ge("score", 80).or_group(|alternative| alternative.eq("tag", "rust"))
    })
    .build()
    .unwrap();
let filter = MetadataFilter::builder()
    .expression(expression)
    .build()
    .unwrap();

let meta = Metadata::new()
    .with("status", "active")
    .with("score", 42_i64)
    .with("tag", "rust");

assert!(filter.matches(&meta));
```

上面的表达式等价于：

```text
status == "active" AND (score >= 80 OR tag == "rust")
```

如果需要对已构造的表达式取反，可以使用 `FilterExpression::try_not()`：

```rust
let expression = FilterExpression::builder()
    .eq("status", "active")
    .build()
    .unwrap()
    .try_not()
    .unwrap();
let filter = MetadataFilter::builder()
    .expression(expression)
    .build()
    .unwrap();
```

缺失键和 `Value::Unset` 使用 fail-closed 三值逻辑。没有具体存储值的比较结果为
unknown；`not` 不会把 unknown 变成 true，AND/OR 会继续传播 unknown，最终只有确定的
true 才会让 `matches()` 返回 `true`。因此 `ne(key, value)` 与
`not(eq(key, value))` 等价，并且都不会匹配缺失值或 unset 值。数值相等、集合成员判断
和范围谓词中的混合数值比较策略由 `NumericComparisonPolicy` 控制。

分组表达式必须至少包含一个谓词。例如 `and_group(|group| group)` 会被 `build()` 拒绝，
因为空分组通常代表调用方构造条件时漏传了约束。

空集合是允许的。`in_set("key", [])` 不匹配任何 metadata 对象。
`not_in_set("key", [])` 只在 `key` 存储具体值时匹配；键缺失或值为 unset 时仍为
unknown，因此不匹配。

schema 校验 filter 时，任意非 NaN 数值表示都与任意数值字段兼容；这项兼容性检查与
运行时比较策略无关。实际匹配时，`Exact` 不经舍入地比较表示出来的数学值。
`Approximate` 会单独排序原生无穷值；有限原生浮点数参与时，它尝试把两个操作数投影为
有限 `f64`，任一操作数无法完成这种投影时回退到精确比较。投影比较取决于当前操作数对
且不满足传递性，因此不得用于排序、分组、实现 `Ord` 或有序键。实际调用
`MetadataSchema::validate(&metadata)` 校验 metadata 时仍然严格：metadata 中存储的值
必须和 schema 声明的具体字段类型一致。

通过 `MetadataFilter::expression()` 和 `FilterExpression::view()` 可以读取完整布尔
结构。只读 view 会暴露 condition、AND、OR、NOT、true、false 节点，但不会暴露私有
构造表示，因此存储 provider 可以安全地把 filter 翻译为自己的查询语言。

### 5) 版本化 filter 序列化格式

`MetadataFilter` 使用严格的 v4 wire format，包含 `version`、`expression`、
`options` 和 `limits` 字段。每个表达式节点使用 `kind` tag；`eq`、`ge`、`in`、
`not_exists` 等条件数据直接内联在节点中，布尔节点使用 `and`、`or`、`not`、
`all` 和 `none`。`options` 中的策略枚举使用 lowercase underscore 值，例如
`exact` 和 `approximate`。未知字段、畸形节点和非 v4 版本都会被拒绝。
`Metadata` 与 `MetadataSchema` 分别使用严格的 v1 envelope，同样拒绝未知字段和
不支持的版本。

## 错误处理

当调用方需要明确区分“键不存在”和“类型不匹配”时，使用 `try_get` 或 schema 校验。
单字段访问返回 `MetadataError`；schema 级校验返回 `MetadataValidationError`，
可以通过 `issues()` 拿到本轮收集到的全部 `MetadataError`。转换诊断会保留结构化
失败原因，但不会嵌入被拒绝的原始值：

```rust
use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataError};

let meta = Metadata::new().with("answer", "forty-two");

match meta.try_get::<i64>("answer") {
    Err(MetadataError::TypeMismatch { expected, actual, .. }) => {
        assert_eq!(expected, DataType::Int64);
        assert_eq!(actual, DataType::String);
    }
    other => panic!("unexpected result: {other:?}"),
}
```

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-metadata = "0.10"
# 使用 schema 数据类型或数值比较策略时需要直接依赖。
qubit-datatype = "0.8"
# 直接构造 Value 操作数时需要依赖。
qubit-value = "0.10"
```

### Feature flags

默认 feature 集只包含核心标量 metadata 支持。请仅启用实际使用的富类型族：

```toml
[dependencies]
qubit-metadata = { version = "0.10", features = ["chrono", "json"] }
```

可用 feature 包括 `chrono`、`big-integer`、`big-decimal`、`big-number`、
`url`、`json` 和 `all`。

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-metadata](https://github.com/qubit-ltd/rs-metadata)
