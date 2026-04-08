# Qubit Metadata

[![CircleCI](https://circleci.com/gh/qubit-ltd/rust-metadata.svg?style=shield)](https://circleci.com/gh/qubit-ltd/rust-metadata)
[![Coverage Status](https://coveralls.io/repos/github/qubit-ltd/rust-metadata/badge.svg?branch=main)](https://coveralls.io/github/qubit-ltd/rust-metadata?branch=main)
[![Crates.io](https://img.shields.io/crates/v/qubit-metadata.svg?color=blue)](https://crates.io/crates/qubit-metadata)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

适用于 Rust 的通用、类型安全、可扩展元数据模型。

## 概述

`qubit-metadata` 提供 `Metadata` 类型——一种结构化键值存储，适用于任何需要向
数据模型附加任意类型、可序列化注解信息的场景。典型使用场景包括：

- 为领域实体（消息、记录、事件等）附加上下文信息
- 在不污染核心模型的前提下携带 provider 或适配器特有字段
- 在服务或库边界之间透传不透明的元数据
- 对带注解的数据（如向量存储记录）表达查询过滤条件

它不是普通的 `HashMap<String, String>`，而是一个以 `serde_json::Value` 为底层
存储、具有类型安全访问接口和完整 `serde` 支持的结构化可扩展点。

## 设计目标

- **类型安全**：通过 `serde` 往返实现基于 `serde_json::Value` 的类型化 get/set API
- **通用性**：不预设任何领域假设，可用于任何 Rust 项目
- **可扩展性**：作为结构化扩展点，而非字符串化的杂物袋
- **序列化**：对 JSON 交换的一流 `serde` 支持
- **过滤**：可选的 `MetadataFilter`，用于可组合的查询条件

## 特性

### 🗂 **`Metadata`**

- 以 `String` 为键、`serde_json::Value` 为值的有序键值存储
- 提供两层类型化访问：便捷的 `get::<T>()` / `set()` 与显式的 `try_get::<T>()` / `try_set()`
- 提供轻量的 JSON 类型检查能力 `MetadataValueKind`
- 支持合并、扩展和迭代操作
- 完整的 `serde` 序列化 / 反序列化支持
- 派生 `Debug`、`Clone`、`PartialEq`、`Default`

### 🔍 **`MetadataFilter`** *（可选，规划中）*

- 用于基于元数据查询的可组合过滤表达式
- 支持等值、范围和逻辑组合器（`and`、`or`、`not`）
- 适用于数据库、向量存储或内存集合中带注解记录的过滤

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-metadata = "0.1.0"
```

## 使用示例

```rust
use qubit_metadata::Metadata;

let mut meta = Metadata::new();
meta.set("author", "alice");
meta.set("priority", 3_i64);
meta.set("reviewed", true);

// 便捷 API：简洁、宽松。
let author: Option<String> = meta.get("author");
assert_eq!(author.as_deref(), Some("alice"));

// 显式 API：保留失败原因。
let priority = meta.try_get::<i64>("priority").unwrap();
assert_eq!(priority, 3);
```

## 错误处理

如果调用方需要区分“键不存在”和“类型不匹配”，应优先使用显式 API，
并检查 `MetadataError`：

```rust
use qubit_metadata::{Metadata, MetadataError, MetadataValueKind};

let mut meta = Metadata::new();
meta.set("answer", "forty-two");

match meta.try_get::<i64>("answer") {
    Err(MetadataError::DeserializationError { actual, .. }) => {
        assert_eq!(actual, MetadataValueKind::String);
    }
    other => panic!("unexpected result: {other:?}"),
}
```

## 许可证

本项目采用 [Apache License 2.0](LICENSE) 许可证。
