# Qubit Metadata

[![CircleCI](https://circleci.com/gh/qubit-ltd/rust-metadata.svg?style=shield)](https://circleci.com/gh/qubit-ltd/rust-metadata)
[![Coverage Status](https://coveralls.io/repos/github/qubit-ltd/rust-metadata/badge.svg?branch=main)](https://coveralls.io/github/qubit-ltd/rust-metadata?branch=main)
[![Crates.io](https://img.shields.io/crates/v/qubit-metadata.svg?color=blue)](https://crates.io/crates/qubit-metadata)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

适用于 Rust 的通用、类型安全且可扩展的元数据模型。

## 概述

`qubit-metadata` 提供 `Metadata` 类型，它是一个结构化键值存储，适合在各种场景下为
数据模型附加任意类型、可序列化的注解信息。常见用法包括：

- 为领域实体（消息、记录、事件等）附加上下文信息
- 在不污染核心模型的前提下携带 provider 或适配器的专有字段
- 基于带注解的数据（如向量存储记录）构建查询过滤条件
- 在服务或库边界之间透传不透明的元数据

它并不是普通的 `HashMap<String, String>`，而是一个面向领域模型的结构化扩展点：
底层使用 `serde_json::Value` 存储，并提供类型安全的访问方式以及一流的 `serde`
支持。

## 设计目标

- **类型安全**：基于 `serde_json::Value` 提供类型化的 get/set API
- **通用性**：不引入领域特定假设，适用于任何 Rust 项目
- **可扩展性**：提供结构化扩展点，而不是把所有内容都塞进字符串字典
- **序列化**：为 JSON 交换提供一流的 `serde` 支持
- **过滤能力**：`MetadataFilter` 用于对 `Metadata` 做可组合的谓词判断

## 特性

### 🗂 **`Metadata`**

- 使用 `String` 作为键、`serde_json::Value` 作为值的有序键值存储
- 提供两层类型化访问接口：简洁的 `get::<T>()` / `set()`，以及显式的 `try_get::<T>()` / `try_set()`
- 可通过 `MetadataValueType` 轻量检查 JSON 值类型
- 支持合并、扩展和迭代
- 完整支持 `serde` 序列化 / 反序列化
- 自动派生 `Debug`、`Clone`、`PartialEq`、`Default`

### 🔍 **`MetadataFilter`**

- 以条件树描述对元数据的约束，通过 `matches(&metadata)` 求值
- 支持比较、存在性、集合成员等叶子条件，以及逻辑与 / 或 / 非组合
- 实现 `Serialize` / `Deserialize`，可将过滤条件以 JSON 形式持久化或跨服务传递
- 适用于内存过滤、检索谓词，或任何需要可移植元数据谓词的场景

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-metadata = "0.1.0"
```

## 用法示例

```rust
use qubit_metadata::Metadata;

let mut meta = Metadata::new();
meta.set("author", "alice");
meta.set("priority", 3_i64);
meta.set("reviewed", true);

// 便捷 API：写法简洁，失败时返回 `None`。
let author: Option<String> = meta.get("author");
assert_eq!(author.as_deref(), Some("alice"));

// 显式 API：保留具体失败原因。
let priority = meta.try_get::<i64>("priority").unwrap();
assert_eq!(priority, 3);
```

## MetadataFilter（元数据过滤）

`MetadataFilter` 表示针对**一份** [`Metadata`](https://docs.rs/qubit-metadata/latest/qubit_metadata/struct.Metadata.html) 是否成立的谓词。先由若干叶子条件与逻辑节点拼成一棵树，再对目标 `Metadata` 调用 [`matches`](https://docs.rs/qubit-metadata/latest/qubit_metadata/enum.MetadataFilter.html#method.matches) 得到 `bool`。

### 叶子构造方法

每个叶子只针对一个键；写入的值与 `Metadata::set` 一样，经 `Serialize` 转为 [`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html) 再参与比较：

| 方法 | 含义 |
|------|------|
| `equal` / `not_equal` | 键对应 JSON 值与给定值相等 / 不相等 |
| `greater` / `greater_equal` / `less` / `less_equal` | 有序比较（数值大小、字符串字典序，以及实现中支持的混合类型规则） |
| `exists` / `not_exists` | 键存在 / 不存在 |
| `in_values` / `not_in_values` | 键的值是否属于给定集合 |

### 逻辑组合

- **`and`** — 所有子过滤器都必须匹配。若当前节点已是 `And`，新条件会**追加**到同一层，避免无谓嵌套。
- **`or`** — 至少一个子过滤器匹配（同样会扁平化追加）。
- **`not`** — 对子树取反。也可通过标准库的 [`Not`](https://doc.rust-lang.org/std/ops/trait.Not.html) 对 `MetadataFilter` 使用 `!filter`。

边界行为：**空的** `And` 对任意 `Metadata` 为真；**空的** `Or` 对任意 `Metadata` 为假。

### 求值

调用 `filter.matches(&meta)` 即可。某个键不存在时，各类叶子条件是否匹配，由 [`Condition`](https://docs.rs/qubit-metadata/latest/qubit_metadata/enum.Condition.html) 的语义决定（例如 `equal` 要求键必须存在；`not_equal` 在键缺失时仍可能为真）。

### 序列化

`MetadataFilter` 实现了 `Serialize` / `Deserialize`，可将过滤条件存入配置、数据库或通过 JSON API 与元数据模型一并交换。

### 示例

**组合 AND — 同时满足：**

```rust
use qubit_metadata::{Metadata, MetadataFilter};

let mut meta = Metadata::new();
meta.set("status", "active");
meta.set("score", 42_i64);

let filter = MetadataFilter::equal("status", "active")
    .and(MetadataFilter::greater_equal("score", 10_i64));

assert!(filter.matches(&meta));
```

**OR、集合成员与取反：**

```rust
use qubit_metadata::{Metadata, MetadataFilter};

let mut meta = Metadata::new();
meta.set("region", "eu");
meta.set("tier", "pro");

let filter = MetadataFilter::in_values("region", ["eu", "us"])
    .or(MetadataFilter::equal("tier", "enterprise"));

assert!(filter.matches(&meta));

let hide_drafts = MetadataFilter::equal("status", "draft").not();
// 等价于：!MetadataFilter::equal("status", "draft")
assert!(hide_drafts.matches(&meta));
```

## 错误处理

如果调用方需要区分“键不存在”和“类型不匹配”，建议优先使用显式 API，
并检查 `MetadataError`：

```rust
use qubit_metadata::{Metadata, MetadataError, MetadataValueType};

let mut meta = Metadata::new();
meta.set("answer", "forty-two");

match meta.try_get::<i64>("answer") {
    Err(MetadataError::DeserializationError { actual, .. }) => {
        assert_eq!(actual, MetadataValueType::String);
    }
    other => panic!("unexpected result: {other:?}"),
}
```

## 许可证

本项目采用 [Apache License 2.0](LICENSE) 授权。
