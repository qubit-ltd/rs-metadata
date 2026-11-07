# qubit-metadata 设计

[English design](design.md) · [中文用户手册](user_guide.zh_CN.md) · [中文 README](../README.zh_CN.md)

本文记录 `qubit-metadata` 0.7 的稳定设计边界，供集成方参考；内容描述长期不变量，
不记录实现过程。

## 目标与非目标

本 crate 提供按 key 排序且带类型的 metadata 容器、可选的 schema/filter 校验，以及严格的 V1
Serde 格式。它不绑定存储 provider，不负责定义 provider 的索引策略，不新增 wire 版本，
也不把诊断格式化承诺为通用保密边界。

## 模块边界

核心 metadata 模块负责按 key 排序的值和类型化读取；schema 模块负责字段声明与校验；filter 模块
负责不可变布尔表达式、匹配选项和接收端限制；wire 模块负责版本化 Serde envelope 与有界
遍历。JSON budget profile 按方向独立配置，解码和编码策略互不替换。

启用 `json` 后，metadata 可以使用有界 JSON 编码；启用 `schema` 后，schema 也提供相同
能力。`json` 和 `filter` 同时启用时，filter 也支持有界编码；这不改变 `schema` 包含
`filter` 的关系。编码器先检查值能否表示以及资源 profile 是否允许，再生成完整的紧凑
JSON 输出。返回 Vec 的 API 会返回该缓冲区；writer API 会先缓冲完整文档再写入，但目标
I/O 出错时仍可能已经写出部分字节。Filter 接收端限制是瞬态策略，不会序列化。持久化、
事务、传输、重试和补偿由应用负责。只有在编码 API 显式接收 `limits.json_encode()` 时，
自定义 `MetadataLimits` 编码 profile 才会生效；解码设置不会改变输出预算。

## 核心数据模型

`Metadata` 将字符串 key 映射到 `qubit_value::Value`，迭代和序列化均按 key 排序。`Value::Unset` 是
已存在的、带声明类型但没有具体值的字段。`MetadataSchema` 区分 required 和 optional 字段，
并按具体 `DataType` 校验已存值。`MetadataFilter` 根据 metadata 计算 `FilterExpression`。
接收端 limits 属于运行时策略，不参与序列化、相等性和哈希。

`get`、`get_ref`、`get_optional` 和 `get_or` 使用值层的类型契约严格读取，全部返回
`MetadataResult`；可选读取只吸收缺失，不隐藏类型不匹配。借用读取不克隆载荷。
`convert`、`convert_with`、`convert_optional_with` 和 `convert_or_with` 显式请求转换，
后几种方法接收策略与限额。默认值复用 `IntoValueDefault`。Rust 会在方法调用前求值参数表达式；只有允许且需要回退时，才调用默认值适配器。例如，`metadata.get_or::<String>("tenant", expensive_default())` 会在调用前执行 `expensive_default()`，无论是否需要回退。传入借用字符串时，只有转换为 owned `String` 的适配工作会延迟到允许且需要回退时执行。若要延迟自定义计算，应将计算放入 `IntoValueDefault` 适配器；这里不表示可以直接传入 `FnOnce` 闭包。
删除旧 `try_get*` 别名和 `get_str`。

Metadata 负责类型化业务属性、schema 校验和条件匹配；配置源组合、作用域、插值和业务结构体
反序列化属于 `rs-config`。特别是 `Config::get` 会转换，`Metadata::get` 要求精确存储类型。

## V1 线协议与版本策略

metadata、schema 和 filter 使用严格的 V1 envelope。未知字段、畸形节点和不支持的版本都会
被拒绝。Filter limits 是接收端的瞬态策略，不会序列化。有界 decoder 将不支持版本报告为
`UnsupportedVersion`；schema envelope 的泛型 Serde 解码拒绝位置式序列，要求规范四字段对象；
现有对象输出和 V1 版本保持不变。schema `fields` map 的每个值也必须是仅含 `data_type` 和
`required` 的对象，拒绝位置式字段数组。若已存 V1 schema 使用字段数组，升级前须用旧版本读取，
再由旧版本重编码为规范对象；新 decoder 不提供数组回退。泛型 Serde 解码仍使用普通
deserializer 错误通道。
filter 的 `options` 同样要求恰含 `numeric_comparison_policy` 的对象；V1 envelope decoder
和直接的 `FilterMatchOptions` 反序列化都会拒绝位置数组。仓库内测试、文档及检索到的下游调用方
均未依赖位置式 options。若调用方存储了这类数组，升级前须使用旧版本读取，再将其重编码为规范对象，
例如 `{"numeric_comparison_policy":"exact"}`（或对应策略）。序列化后的 V1 输出和 wire 版本不变。

## 资源与信任边界

完整的不可信 JSON 输入应使用带 `MetadataLimits` 的 slice decoder。输入字节预算在解析前检查；
随后由共享 JSON session 统计通用结构和 payload，再由 metadata、schema 或 filter seed 统计领域资源。
默认 profile 包含 byte、depth、node、sequence/map、string/key/number/payload 以及领域限制。
内存中的 metadata 本身不设这些 wire 上限；V1 wire map 会执行规范的条目数和 key 长度限制。

### 专用 metadata JSON helper

专用 metadata slice decoder 首先在共享 JSON 预算下物化无重复键的 JSON 源树，再应用 wire
seed 并将值转换为 metadata 领域表示。启用 `serde_json/arbitrary_precision` 时，此路径仍保留
原生数字事件。被拒绝的 charge 不应用；失败时已接受的输入字节计费保留，源值计费则在 wire
seed 成功后、后续领域转换前提交。领域反序列化错误可能不再保留原始源码坐标。当前
unique-key decoder 没有额外固定的 127 层容器上限，但仍受配置的通用 JSON 深度预算约束。
这些属于实现细节，不是 API 或性能保证。通用 Serde seed 仍依赖外层 deserializer 提供的事件。

专用 filter JSON decoder 首先在共享 JSON 解码预算下扫描并物化完整的唯一键 JSON 源树，然后
由领域 seed 在构造 filter 时检查接收方 AST 和 membership 限制。收紧 `FilterLimits` 不会停止
源树物化，因此应同时配置通用 JSON budget 和领域限制。该物化契约只适用于专用 JSON decoder，
不适用于任意 `serde::Deserialize` deserializer。

## 三值过滤语义

对缺失 key 或 unset 值进行相等、范围和集合比较会得到 unknown。具体值无法比较或数值比较失败时也会
得到 unknown；不同类型的值不会隐式转换，数值类型是否兼容由 `NumericComparisonPolicy` 决定。取反会
保留 unknown，因此 `ne`、否定比较和 `not_in_set` 不能把 unknown 变为匹配。存在性谓词返回确定的布尔值：
没有存储具体值时，`exists` 为 false、`not_exists` 为 true，因此 `exists(...).not()` 为 true。
`matches` 只有在结果明确为 true 时才返回 true。布尔组合遵循确定值优先（`false AND unknown` 为 false，
`true OR unknown` 为 true），其余混合情况保持 unknown。集合仍按候选逐项比较，并遵循确定结果优先规则。
数值比较策略需要显式配置；近似比较不适合用于排序或分组。

## 错误分类与脱敏

`MetadataError` 覆盖 metadata 读取和 schema 校验，校验结果可以暴露多个相互独立的问题。Filter
builder 报告畸形或不兼容表达式。Wire 错误区分领域限制、语法/envelope 失败和不支持版本。有界 metadata 与 schema 编码通过 `MetadataWireEncodeError::Domain` 保留 `MetadataError::WireLimitExceeded`；schema wire preflight 优先于 JSON 输出预算。原先针对 schema wire 限额匹配 `Json` 的调用方应改为匹配 `Domain` 并读取 `kind`、`value` 和 `maximum`；普通 Serde 序列化错误仍使用 `Json`。诊断
保留有用的类型、数量和版本号，但不嵌入被拒绝的 metadata 值；这属于诊断脱敏，不承诺隐藏任意
用户 key 或错误文本。复合表达式、filter 和 schema 诊断共用一个累计资源预算；超出预算时使用
不完整 marker。

`MetadataWireDecodeError::Decode` 保留共享 decoder 对空输入、非法 UTF-8、顶层类型不符和
反序列化失败的分类，包括 `kind()` 与结构化 accessor。`Read`、`Budget`、`Quantity` 和
`Syntax` 继续使用原有的独立映射。标准 metadata 入口采用 `Redacted` 诊断；只有调用方在
转换前选择了 `Detailed`，才会保留详细来源。`InvalidJson` 变体仍可手工构造；标准 metadata
slice 解码入口也会在 wire 物化后的 payload 无法转换为 scalar metadata 值时返回该变体，例如
条目包含 collection payload。此转换发生在 wire 物化后，可能已不再保留原始源坐标。若调用方
原先将它视为 JSON 失败的完整集合，应增加 `Decode` 分支。展示文本和来源链深度不属于兼容性
契约。

`MetadataError::ValueAccess` 保存 boxed `ValueError`。缺失读取保留 `ValueMissing` 的原因、
源/目标类型、可选集合索引和原始转换错误，`Error::source` 保留这条来源链。
删除 `MetadataError::MissingValue`，schema 的 `TypeMismatch` 仍是独立领域错误。
可选/默认值读取使用对应的值缺失 predicate，不吞掉所有错误。

## Feature 与依赖架构

默认 feature 集仅包含核心能力。`filter` 提供 filter 所需依赖，`schema` 包含 `filter`，`json`
提供有界 JSON 支持及 JSON budget 集成；各类值类型独立由 feature 控制。应用应按边界只启用实际
需要的能力层。

## 兼容策略

0.6 曾按上述规则修改读取签名和缺失错误处理，不保留兼容别名。V1 wire 表示、filter 语义和
schema 校验行为保持不变。已存值的 schema 校验仍与兼容数值的 filter 检查分离。
集成方应使用 builder 和公开 Serde API，让 wire 数据保持在规范限制内，并把未来 wire 版本变化
作为明确的兼容事件处理。
