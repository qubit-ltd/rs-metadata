# 从 0.6 升级到 0.7

[English migration guide](migration_0.7.md) · [用户手册](user_guide.zh_CN.md) · [README](../README.zh_CN.md) · [设计文档](design.zh_CN.md) · [API 文档](https://docs.rs/qubit-metadata)

本文说明升级到 `qubit-metadata` 0.7 时如何协调依赖并核查当前目标版本的兼容契约。下表以 Cargo
registry 中已发布的 `qubit-metadata 0.6.2` 清单和本仓库 `0.7.0` 清单为对照，
不能据此推断所有 0.6.x 版本都使用相同依赖。

## 升级前的兼容性核查

当应用保存、发送或分类 metadata wire 数据时，请按此清单核查。这里描述的是当前目标版本契约；各项输入或错误契约变化首次出现在哪个版本尚未确认。稳定设计文档说明了 [V1 wire 契约](design.zh_CN.md#v1-线协议与版本策略)和[错误分类](design.zh_CN.md#错误分类与脱敏)。

| 核查项 | 适用条件 | 当前契约与行动 |
| --- | --- | --- |
| Schema envelope 与字段对象 | 应用持久化或生成 schema JSON，尤其是数据可能含位置序列时 | envelope 必须是规范四字段对象，且 `fields` 中每个值必须是仅含 `data_type` 和 `required` 的对象。位置式 envelope 序列和位置式字段数组都会被拒绝。若已存数据使用其中任一形式，先备份，再用已验证确实接受该表示的旧版读取，编码为规范对象，由目标版本验证，最后切换消费者。目标版验证成功前保留备份。见 [schema wire 契约](design.zh_CN.md#v1-线协议与版本策略)。 |
| Filter `options` | 应用持久化/发送 filter JSON 或直接反序列化 `FilterMatchOptions`，且 options 可能是位置数组时 | `options` 必须是仅含 `numeric_comparison_policy` 的对象；envelope decoder 和直接 options 反序列化都会拒绝位置数组。编码为规范对象时保留原有策略，不能一律替换成 `exact`。旧数组按“备份→经验证的旧版读取→规范对象编码→目标版验证→切换消费者”处理。见 [filter options 契约](design.zh_CN.md#v1-线协议与版本策略)。 |
| Decode 错误分类 | 调用方把 `InvalidJson` 当作全部 JSON 解码失败时 | `Decode` 承载共享 decoder 失败，如空输入、非法 UTF-8、顶层类型不符和反序列化失败，并提供结构化 accessor。应增加 `Decode` 处理，同时保留独立的 `Read`、`Budget`、`Quantity`、`Syntax` 分支及现有 `InvalidJson` 分支。见 [解码错误契约](design.zh_CN.md#错误分类与脱敏)。 |
| 有界编码的 wire 限额 | 调用方只通过编码错误的 `Json` 分支处理 metadata/schema wire 限额时 | 有界 metadata 和 schema encoder 将 wire 限额报告为 `Domain(MetadataError::WireLimitExceeded { .. })`；应处理此变体及其 `kind`、`value`、`maximum`。普通 Serde 序列化失败仍属于 `Json`。见[有界编码错误契约](design.zh_CN.md#错误分类与脱敏)。 |

位置数组相关变化收紧了输入接受范围，不改变规范 V1 输出或 wire 版本。不要推定旧版本一定接受某种存储表示；迁移前应使用准确的库版本和数据形态进行核实。

## 依赖版本对照

| crate | 0.6.2 要求 | 0.7.0 要求 |
| --- | --- | --- |
| `qubit-datatype` | `0.14` | `^0.15` |
| `qubit-value` | `0.12` | `^0.13` |
| `qubit-budget` | `0.7` | `^0.10` |
| `qubit-json` | `0.10` | `^0.12` |
| `qubit-redact` | `0.9` | `^0.10` |

这些版本来自两份清单声明。crate 还依赖 `serde` 和可选的 `qubit-utils`，
它们不属于本次版本对照。Cargo 会在各自兼容范围内解析具体补丁版本。

## 只对自己公开使用的类型声明直接依赖

如果应用只调用 `qubit-metadata` 的 API，更新该依赖后让 Cargo 解析传递依赖即可。
当应用代码直接引用或构造某个 Qubit crate 的公开类型时，再声明对应的直接依赖。
例如，schema 字段使用 `qubit_datatype::DataType`；启用 schema 并自行构造字段时，
应让直接依赖与 0.7 对齐：

```toml
[dependencies]
qubit-metadata = { version = "0.7", features = ["schema", "json"] }
qubit-datatype = "0.15"
```

如果代码会构造 `qubit_value::Value`，再添加 `qubit-value = "0.13"`。
如果代码要引用 JSON 预算类型，再添加
`qubit-budget = { version = "0.10", features = ["json"] }`。如果应用直接命名或公开
某个依赖 crate 的错误类型，也应对齐该 crate 的直接依赖版本。不要只因表中列出
`qubit-json` 或 `qubit-redact` 就把它们加成直接依赖；除非应用自身使用其 API 或错误类型。

当类型需要穿过函数边界时，直接依赖的版本尤其重要。例如，某个库在公开签名中
暴露 `Value`，其调用方就必须与它使用相同的 `qubit-value` crate 身份。Cargo 允许
依赖图中同时存在多个版本，但不同版本里的同名 Rust 类型并不相同，不能直接互传。
升级共享公开签名的所有者时，也要协调调用方。如果短期内必须隔离版本，应在明确的
应用边界转换数据；不能把 Serde 当作任意版本之间都无损的通用桥接。

## 检查 Cargo 实际解析结果

修改 `Cargo.toml` 后，可以查看重复依赖以及 `qubit-metadata` 的依赖路径：

```bash
cargo tree -d
cargo tree -i qubit-metadata@0.7.0
```

反向查询必须使用 Cargo 实际选中的版本；如果锁文件解析到另一个 0.7 补丁版本，
相应替换 `0.7.0`。若 `qubit-value`、`qubit-datatype` 或 `qubit-budget` 的类型会穿过
应用自己的 API，就应检查这些 crate 是否出现多个版本。重复本身不一定是错误；真正需要
处理的是不同版本的类型必须跨越同一个有类型约束的边界。

## 将 0.6 API 迁移与本次依赖升级区分开

本文不把 0.6 已经引入的 API 变更重新归到 0.7。如果你从早于 0.6 的版本升级，或者
代码仍在使用已移除的 `try_get*`、`get_str`、`MetadataError::MissingValue`，还需阅读
[迁移到 0.6](migration_0.6.zh_CN.md)。0.7 的 feature 选择、使用流程和错误处理见
[用户手册](user_guide.zh_CN.md)。

版本表说明的是清单中的依赖要求，并不表示每个下游项目都要把五个 crate 全部列为直接
依赖。只为应用实际引用的类型声明直接依赖，并在跨模块或跨库的接口处核对类型版本。
