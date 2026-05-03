# Qubit Metadata

[![CircleCI](https://circleci.com/gh/qubit-ltd/rs-metadata.svg?style=shield)](https://circleci.com/gh/qubit-ltd/rs-metadata)
[![Coverage Status](https://coveralls.io/repos/github/qubit-ltd/rs-metadata/badge.svg?branch=main)](https://coveralls.io/github/qubit-ltd/rs-metadata?branch=main)
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

## 设计目标

- **类型明确**：用 `qubit_value::Value` 保留具体运行时类型。
- **构造方便**：同时支持可变的 `set()` 和链式的 `with()`。
- **可选 schema**：用 `MetadataSchema` 校验字段名、必填字段和具体 `DataType`。
- **过滤器 builder**：通过 builder 构造不可变的 `MetadataFilter`。
- **序列化友好**：metadata、schema、filter 都支持 `serde`，便于配置、存储和跨服务传输。

## 特性

### 1) 类型化 metadata 容器

`Metadata` 是有序的 `String -> Value` 映射，支持 `get`、`set`、`try_get`、
带 schema 校验的 `set_checked` / `with_checked`、链式 `with`、迭代、合并、
保留和 `BTreeMap<String, Value>` 转换。

```rust
use qubit_metadata::Metadata;

let meta = Metadata::new()
    .with("author", "alice")
    .with("priority", 3_i64)
    .with("reviewed", true);

assert_eq!(meta.get::<String>("author").as_deref(), Some("alice"));
assert_eq!(meta.try_get::<i64>("priority").unwrap(), 3);
```

### 2) 用 schema 做校验和存储规划

`MetadataSchema` 使用 `qubit_datatype::DataType`。当存储后端要求预先声明 metadata 字段时，
schema 可以直接作为字段定义来源；在构造 filter 时，也可以提前校验字段、操作符和
过滤值类型是否匹配。`UnknownFieldPolicy` 同时作用于 metadata 校验和 filter 校验：
已声明字段仍然严格校验，未知 filter 字段只有在策略为 `Allow` 时才会被接受。

```rust
use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataSchema};

let schema = MetadataSchema::builder()
    .required("tenant_id", DataType::String)
    .required("score", DataType::Int64)
    .optional("source", DataType::String)
    .build();

let meta = Metadata::new()
    .with("tenant_id", "acme")
    .with("score", 42_i64);

schema.validate(&meta).unwrap();
```

### 3) builder 构造不可变 filter

`MetadataFilter::builder()` 返回 builder，调用 `build()` 后得到
`Result<MetadataFilter, MetadataError>`。这样空分组这类结构非法的表达式会明确报错，
不会静默变成 no-op。如果已有 schema，可以用 `build_checked(&schema)` 在构建时校验字段
是否存在、操作符是否适用于字段类型、过滤值类型是否兼容。

```rust
use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataFilter, MetadataSchema};

let schema = MetadataSchema::builder()
    .required("status", DataType::String)
    .required("score", DataType::Int64)
    .build();

let filter = MetadataFilter::builder()
    .eq("status", "active")
    .and_ge("score", 10)
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
| `exists` / `not_exists` | 键存在 / 不存在 |
| `in_set` / `not_in_set` | 集合包含 / 排除 |
| `and_*` / `or_*` | 用明确连接词追加一个谓词 |
| `and` / `or` / `and_not` / `or_not` | 追加分组子表达式 |
| `not` | 对当前表达式取反 |

分组子表达式使用闭包构造。闭包会收到一个新的 builder，闭包返回的表达式会作为一个
整体追加到外层表达式中：

```rust
use qubit_metadata::{Metadata, MetadataFilter};

let filter = MetadataFilter::builder()
    .eq("status", "active")
    .and(|g| g.ge("score", 80).or_eq("tag", "rust"))
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

如果整个分组需要取反，可以使用 `and_not` 或 `or_not`：

```rust
let filter = MetadataFilter::builder()
    .eq("status", "active")
    .and_not(|g| g.ge("score", 80).or_eq("tag", "java"))
    .build()
    .unwrap();
```

负向谓词遇到缺失键时的行为由 `MissingKeyPolicy` 控制。数值相等、集合成员判断和范围
谓词中的混合数值比较策略由 `NumberComparisonPolicy` 控制。

分组表达式必须至少包含一个谓词。例如 `and(|g| g)` 和 `or_not(|g| g)` 会被
`build()` 拒绝，因为空分组通常代表调用方构造条件时漏传了约束。

空集合是允许的。`in_set("key", [])` 不匹配任何 metadata 对象。
`not_in_set("key", [])` 在 `key` 存在时匹配；如果 `key` 缺失，则和其他负向谓词一样，
遵循当前配置的 `MissingKeyPolicy`。

schema 校验 filter 时，所有数值型 `DataType` 之间视为兼容。这是有意放松：
调用方构造过滤条件时经常只能给出方便的数值字面量，未必能精确匹配存储字段的具体
整数/浮点类型。实际调用 `MetadataSchema::validate(&metadata)` 校验 metadata 时仍然严格：
metadata 中存储的值必须和 schema 声明的具体字段类型一致。

### 5) 版本化 filter 序列化格式

`MetadataFilter` 序列化后使用明确的 wire format，包含 `version`、`expr` 和
`options` 字段。表达式节点使用 `type`，条件节点使用稳定的 `op` 操作符名，例如
`eq`、`ge`、`in` 和 `not_exists`。单独序列化 `Condition` 时也使用同一套条件
wire 表示。内部表达式树不属于序列化契约。新的序列化输出始终使用版本化格式。

## 错误处理

当调用方需要明确区分“键不存在”和“类型不匹配”时，使用 `try_get` 或 schema 校验：

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
qubit-metadata = "0.4.0"
```

## 许可证

本项目采用 [Apache License 2.0](LICENSE) 授权。
