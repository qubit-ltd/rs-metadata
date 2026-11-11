# 从 0.6 升级到 0.7

[English migration guide](migration_0.7.md) · [用户手册](user_guide.zh_CN.md) · [README](../README.zh_CN.md) · [设计文档](design.zh_CN.md) · [API 文档](https://docs.rs/qubit-metadata)

本文说明升级到 `qubit-metadata` 0.7 时如何协调依赖并核查当前目标版本的兼容契约。运行时
对照使用 registry 中的 `qubit-metadata 0.6.2` 与本仓库 `0.7.0` checkout；不能据此推断
所有 0.6.x 版本的行为，也不能确定某项契约首次变化的版本。

## 升级前的兼容性核查

当应用保存、发送或分类 metadata wire 数据时，请核对下表。表中记录的是精确
0.6.2 到 0.7.0 的实测结果，不用于推断各差异首次出现的版本。稳定设计文档说明了
[V1 wire 契约](design.zh_CN.md#v1-线协议与版本策略)和[错误分类](design.zh_CN.md#错误分类与脱敏)。

| 用例 / API | registry 0.6.2 实测 | 当前 0.7.0 实测 | 升级影响与迁移方式 |
| --- | --- | --- | --- |
| schema_object / MetadataSchema::decode_json_slice | 接受；输出规范对象 | 接受；输出规范对象 | 未观察到输入形态变化。 |
| schema_envelope_sequence / schema decoder | 拒绝，分类为 InvalidJson | 拒绝，结构化分类为 Decode | 接受范围未变化；调用方需考虑错误分类变化。 |
| schema_field_sequence / schema decoder | 接受；规范化为对象字段 | 拒绝，结构化分类为 Decode | 先备份，再由精确版本 0.6.2 的消费者读取，将 schema 序列化为规范对象（实测输出保留 name 字段、string 类型、required: true 和两个 reject 策略）；由 0.7 校验后再切换消费者。 |
| options_object / 直接 FilterMatchOptions | 接受 approximate | 接受 approximate | 未观察到输入形态变化。 |
| options_exact_object / 直接 FilterMatchOptions | 接受 exact | 接受 exact | 未观察到输入形态变化；探针同时覆盖 exact 和 approximate 策略。 |
| options_sequence / 直接 FilterMatchOptions | 接受；规范化为保留 approximate 的对象 | 直接 Serde 反序列化拒绝 | 由 0.6.2 读取并序列化成 options 对象，保留其策略，再由 0.7 校验。不要把所有策略改成 exact。 |
| filter_options_sequence / filter decoder | 接受；规范化后的 filter 保留 approximate | 拒绝，结构化分类为 Decode | 备份后由 0.6.2 读取，将 filter 序列化为 options 对象，由 0.7 校验，再切换消费者。 |
| empty / 有界 metadata、schema、filter decoder | 拒绝，分类为 Syntax | 拒绝，分类为 Syntax | 未观察到分类变化。 |
| invalid_utf8 / 有界 metadata、schema、filter decoder | 拒绝，分类为 InvalidJson（共享 decoder 将其归为非法 UTF-8） | 拒绝，结构化分类为 Decode | 增加 Decode 分支，使用结构化 kind，不匹配显示文本。 |
| wrong_top_level / 有界 metadata、schema、filter decoder | 拒绝，分类为 InvalidJson | 拒绝，结构化分类为 Decode | 输入仍被拒绝；调用方需考虑增加 Decode 分支。 |
| invalid_syntax / 有界 metadata、schema、filter decoder | 拒绝，分类为 Syntax | 拒绝，分类为 Syntax | 未观察到分类变化。 |
| 4,097 个 metadata 条目或 schema 字段 / 有界编码 | 拒绝，分类为 Json(CustomSerialization) | 拒绝，分类为 Domain(MetadataError::WireLimitExceeded { kind: Entries, value: 4097, maximum: 4096 }) | 处理 Domain 并读取结构化限额事实；0.6.2 的公开编码错误不提供这些事实。 |
| 257 字节 metadata 或 schema key / 有界编码 | 拒绝，分类为 Json(CustomSerialization) | 拒绝，分类为 Domain(MetadataError::WireLimitExceeded { kind: KeyBytes, value: 257, maximum: 256 }) | 处理 Domain 并读取结构化限额事实；0.6.2 的公开编码错误不提供这些事实。 |

旧消费者会把实际接受并序列化的输出写入
`tests/fixtures/compatibility/normalized/`；当前测试再读取这些相同字节，解码并比较 schema
字段、required 标记、unknown-field 策略和 numeric 策略。对照使用独立消费者
[tests/fixtures/compatibility-0.6.2/](../tests/fixtures/compatibility-0.6.2/)
和两边按字节共享的输入
[tests/fixtures/compatibility/](../tests/fixtures/compatibility/)。当前版断言位于
[compatibility_contract_tests.rs](../tests/compatibility_contract_tests.rs)。生成旧消费者锁文件后，
以下命令均以退出码 0 完成：

```sh
cargo run --locked --manifest-path tests/fixtures/compatibility-0.6.2/Cargo.toml
cargo tree --manifest-path tests/fixtures/compatibility-0.6.2/Cargo.toml -i qubit-metadata@0.6.2
cargo test --all-features --test compatibility_contract_tests -- --nocapture
```

旧消费者启用 schema,json；锁文件解析到 registry qubit-metadata 0.6.2、
qubit-datatype 0.14.0、qubit-value 0.12.2 和 0.6 系列 JSON crate。当前测试针对
仓库 checkout 的 0.7.0，并启用全部 feature。锁文件与源码是可复现的版本和依赖证据；
已验证边界仅为 0.6.2 → 本次 0.7.0 checkout。

已实测的位置数组输入变化不影响规范 V1 输出或 wire 版本。证据确认 0.6.2 接受位置式
schema 字段值和位置式 filter options，但拒绝位置式 schema envelope。不能据此推断其他
0.6.x 版本行为，也未确定首次改变接受范围的版本。

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
