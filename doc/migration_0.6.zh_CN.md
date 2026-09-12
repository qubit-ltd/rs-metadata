# 迁移到 0.6：明确读取方式与边界诊断

[English migration guide](migration_0.6.md) · [用户手册](user_guide.zh_CN.md) · [设计文档](design.zh_CN.md)

本文面向从更早版本升级到 `qubit-metadata` 0.6 的团队。它在[用户手册](user_guide.zh_CN.md)
核心工作流之外，补充 0.6 中变更的 API 命名、读取语义以及结构化 wire 诊断说明。

## 明确读取 API

索引流程中，校验标识字段时通常需要保持原有类型；接收外部文本时，才显式请求转换：

```rust
use qubit_metadata::Metadata;
let metadata = Metadata::new().with("port", "8080").with("count", 3_i64);
assert_eq!(metadata.get::<i64>("count").unwrap(), 3);
assert!(metadata.get::<i64>("port").is_err());
assert_eq!(metadata.convert::<u16>("port").unwrap(), 8080);
assert_eq!(metadata.get_ref::<str>("port").unwrap(), "8080");
```

`get` 使用 `StrictValueRead`，`get_ref::<str>` 借用存储文本。`convert_with` 接收
`ConversionPolicy` 和 `ConversionLimits`，每次调用使用独立预算。
`convert_optional_with` 和 `convert_or_with` 为可选/默认值读取显式应用同一策略。
`get_or` 严格读取，仅缺失键或类型符合要求的 unset 才通过 `IntoValueDefault` 适配默认值。
转换默认值还允许标量被策略判定缺失的情况；非法转换不会触发默认值。

与 `Config::get` 不同，`Metadata::get` 不会隐式转换。0.6 的 `get` 改为可能失败的严格读取。
允许缺失时使用 `get_optional` 并传播 `Result`；需要按策略转换时使用
`convert_optional_with`。非法值返回错误，不再折叠为 `None`。

## 读取与缺失值诊断

读取错误把 `ValueError` 保存在 `MetadataError::ValueAccess` 中，通过 `Error::source`
保留来源。`ValueError::Missing` 内的 `ValueMissing` 提供 `reason()`、`source_type()`、
`target_type()`、`source_index()` 和可能存在的原始转换错误，不应把这些事实压成文本。

schema 的 `MetadataError::TypeMismatch` 仍是独立的 schema 错误。Metadata 的 filter
feature 和 fail-closed 匹配语义保持不变。

## API 与错误对照

| 原调用或错误 | 新用法 | 行为变化 |
| --- | --- | --- |
| 返回 `Option` 的 `get::<T>(key)` | `get_optional::<T>(key)` | 返回 `Result<Option<T>>`，严格检查类型 |
| 转换读取 `try_get::<T>(key)` | `convert::<T>(key)` | 显式表达转换意图 |
| `try_get_strict::<T>(key)` | `get::<T>(key)` | 常规 getter 默认严格读取 |
| `get_str` / `try_get_str` | `get_ref::<str>` | 可能失败的借用严格读取 |
| `try_convert` / `try_convert_with` | `convert` / `convert_with` | 不保留兼容别名 |
| 转换型 `get_or` | `convert_or_with` | 显式策略与限额，非法值继续报错 |
| `MetadataError::MissingValue` | 包含 `ValueError::Missing` 的 `ValueAccess` | 保留缺失事实和来源链 |

## Wire 解码诊断

有界 metadata/schema JSON 解码遇到条目数或键长超限时，返回
`MetadataWireDecodeError::Domain(MetadataError::WireLimitExceeded { .. })`；
不支持的版本返回 `UnsupportedVersion { expected, actual }`。
这些结构只包含数量和版本号，不包含元数据值。语法及其他严格信封错误继续使用默认脱敏诊断。
V1 序列化格式保持不变。

## Filter builder 快速失败语义

过滤器 builder 在构造阶段拒绝超限输入。集合迭代器最多被消费 129 次：128 项有效容量加
1 项超限检测，检测项不会执行值转换。首次失败后，后续操作不再转换值、不再消费集合，
也不再调用分组闭包；Rust 仍会正常求值调用参数。每次组合（包括嵌套分组）都会检查节点数和深度。

## 下游集成注意点

跨模块字段应明确具体类型、单位和缺失规则。存储 schema 适配器必须拒绝无法表达的约束；
通过逻辑 schema 校验并不意味着后端支持所有运算符。
`merge` 仍会覆盖同名键；若批次中的请求标识等字段可能冲突，聚合层应单独保留逐批元数据。
