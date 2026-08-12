# Qubit Metadata

[![Rust CI](https://github.com/qubit-ltd/rs-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-metadata.svg?color=blue)](https://crates.io/crates/qubit-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-metadata` 是面向 Rust 应用的类型明确、有序 metadata 模型，适合在不削弱
核心数据模型类型的前提下附加可扩展字段。它在一个简洁的 API 中提供类型转换读取、
可选 schema、可组合 filter 以及严格的 Serde wire format。

## 一个真实场景

假设文档处理流水线会把分片发送到向量数据库。分片类型可以把稳定字段保留在自身的
结构体中，把随数据来源或索引后端变化的字段放入 `Metadata`：

```rust
use qubit_metadata::Metadata;

let metadata = Metadata::new()
    .with("tenant_id", "acme")
    .with("document_id", "doc-42")
    .with("chunk_index", 3_i64)
    .with("language", "en");

let tenant: Option<String> = metadata.get("tenant_id");
assert_eq!(tenant.as_deref(), Some("acme"));
assert_eq!(metadata.try_get::<i64>("chunk_index").unwrap(), 3);
```

存储值通过 `qubit_value::Value` 保留具体运行时类型。当后端需要固定字段和查询校验时，
可以增加 `MetadataSchema`，并据此构造 `MetadataFilter`。完整场景以及错误诊断、wire
限制见[中文用户手册](doc/user_guide.zh_CN.md)。

## 为什么需要这个项目

普通 map 虽然方便，却会把许多重要决策分散给每个调用方：字段允许什么值、字段是否必填、
filter 如何分组，以及不可信序列化输入如何限制。本 crate 集中表达这些决策，同时让核心
metadata 容器不依赖任何存储 provider 或具体领域模型。

## 安装

```toml
[dependencies]
qubit-metadata = "0.10"
```

默认 feature 集只提供核心 metadata 容器。需要 schema 校验时启用 `schema`；它会包含
`filter`：

```toml
[dependencies]
qubit-metadata = { version = "0.10", features = ["schema"] }
```

可选 feature 包括 `chrono`、`big-integer`、`big-decimal`、`big-number`、`url`、`json` 和
`all`。声明 schema 字段类型时直接依赖 `qubit-datatype`；直接构造 `Value` 操作数时依赖
`qubit-value`；定制定向 JSON limits 时直接依赖 `qubit-budget`。

## 提供的能力

- `Metadata`：有序的 `String -> Value` 存储，支持 `get`、带诊断的 `try_get`、`set`、
  `insert`、`with`、迭代、合并和 schema 校验写入。
- `MetadataSchema`：必填/可选字段定义、具体 `qubit_datatype::DataType` 校验，以及相互
  独立的未知 metadata 字段和未知 filter 字段策略。
- `FilterExpression` 与 `MetadataFilter`：不可变布尔表达式，支持相等、范围、集合、存在性、
  分组、取反、匹配选项和接收端表达式资源限制。
- metadata、schema、filter 的严格 V1 Serde 格式。启用 `json` 后，还可使用带资源限制的
  JSON slice 解码。
- 结构化的 `MetadataError`、校验错误和 wire 解码错误，帮助调用方区分键缺失、unset 值、
  类型不匹配、非法表达式以及输入限制失败。

解码和编码策略刻意保持方向独立。例如，接收端可以收紧输入准入，而不意外改变输出额度：

```rust
use qubit_json::JsonResource;
use qubit_budget::ResourceLimit;
use qubit_metadata::default_json_decode_limits;
use qubit_metadata::default_json_encode_limits;
use qubit_metadata::MetadataLimits;

let decode = default_json_decode_limits().with_input_bytes_limit(
    ResourceLimit::new(JsonResource::InputBytes, 64 * 1024),
);
let encode = default_json_encode_limits().with_output_bytes_limit(
    ResourceLimit::new(JsonResource::OutputBytes, 128 * 1024),
);
let limits = MetadataLimits::default()
    .with_json_decode(decode)
    .with_json_encode(encode);
```

`decode_json_slice_with_limits` 会根据 decode profile 创建一个 `JsonDecodeSession`，
让它同时负责完整输入准入和整个 seed wire 遍历；encode profile 则单独交给有界序列化
边界。失败请求不会消费被拒绝的 charge，但本次操作此前已接受的消耗不会回滚。

## 重要边界

- `get` 会有意把键缺失和转换失败都折叠为 `None`；需要判断具体原因时使用 `try_get`。
- `Value::Unset` 会记录声明类型，但不是具体值。required schema 字段会拒绝它，filter
  谓词也不会将它视为匹配。
- filter 使用 fail-closed 三值逻辑：unknown 不会通过取反变成匹配。
- 存储 metadata 的 schema 校验仍严格要求具体字段类型；filter 的 schema 检查则允许兼容的
  数值表示。
- 默认 JSON 解码会限制输入字节数、通用 JSON 结构与 payload，以及 metadata 条目数、schema 字段数和
  key 长度。领域限制不能超过 V1 序列化的规范硬上限。Filter limits 是接收端瞬态策略，不会
  写入 V1 wire；共享 JSON adapter 负责通用遍历，filter seed 负责 AST 和 membership 领域限制。
- 脱敏后的 `Debug` 和 `Display` 适合诊断，不应被当成任意用户 key 或错误文本的保密边界。
- Metadata key 本身是普通字符串。当 key 跨越模块、provider 或存储边界时，应在所属边界定义
  唯一的字符串常量，并在读写时复用；如果还需要校验 key/value 契约，应使用
  `MetadataSchema`。

## 延伸阅读

- [中文用户手册](doc/user_guide.zh_CN.md)
- [English User Guide](doc/user_guide.md)
- [Rust API 文档](https://docs.rs/qubit-metadata)
- [English README](README.md)

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
