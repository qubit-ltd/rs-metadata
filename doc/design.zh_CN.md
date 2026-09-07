# qubit-metadata 设计

[English design](design.md) · [中文用户手册](user_guide.zh_CN.md) · [中文 README](../README.zh_CN.md)

本文记录 `qubit-metadata` 0.10 的稳定设计边界，供集成方参考；内容描述长期不变量，
不记录实现过程。

## 目标与非目标

本 crate 提供有序且带类型的 metadata 容器、可选的 schema/filter 校验，以及严格的 V1
Serde 格式。它不绑定存储 provider，不负责定义 provider 的索引策略，不新增 wire 版本，
也不把诊断格式化承诺为通用保密边界。

## 模块边界

核心 metadata 模块负责有序值和类型化读取；schema 模块负责字段声明与校验；filter 模块
负责不可变布尔表达式、匹配选项和接收端限制；wire 模块负责版本化 Serde envelope 与有界
遍历。JSON budget profile 按方向独立配置，解码和编码策略互不替换。

## 核心数据模型

`Metadata` 将字符串 key 映射到 `qubit_value::Value`，并保留插入顺序。`Value::Unset` 是
已存在的、带声明类型但没有具体值的字段。`MetadataSchema` 区分 required 和 optional 字段，
并按具体 `DataType` 校验已存值。`MetadataFilter` 根据 metadata 计算 `FilterExpression`。

## V1 线协议与版本策略

metadata、schema 和 filter 使用严格的 V1 envelope。未知字段、畸形节点和不支持的版本都会
被拒绝。Filter limits 是接收端的瞬态策略，不会序列化。有界 decoder 将不支持版本报告为
`UnsupportedVersion`；泛型 Serde 解码仍使用普通 deserializer 错误通道。

## 资源与信任边界

完整的不可信 JSON 输入应使用带 `MetadataLimits` 的 slice decoder。输入字节预算在解析前检查；
随后由共享 JSON session 统计通用结构和 payload，再由 metadata、schema 或 filter seed 统计领域资源。
默认 profile 包含 byte、depth、node、sequence/map、string/key/number/payload 以及领域限制。
内存中的 metadata 本身不设这些 wire 上限；V1 wire map 会执行规范的条目数和 key 长度限制。

## 三值过滤语义

谓词结果分为 true、false 和 unknown。缺失 key 与 unset 值都是 unknown。`matches` 只有在结果
明确为 true 时才返回 true；布尔组合会传播 unknown，取反也不会把 unknown 变成 true。数值比较
策略需要显式配置；近似比较不适合用于排序或分组。

## 错误分类与脱敏

`MetadataError` 覆盖 metadata 读取和 schema 校验，校验结果可以暴露多个相互独立的问题。Filter
builder 报告畸形或不兼容表达式。Wire 错误区分领域限制、语法/envelope 失败和不支持版本。诊断
保留有用的类型、数量和版本号，但不嵌入被拒绝的 metadata 值；这属于诊断脱敏，不承诺隐藏任意
用户 key 或错误文本。

## Feature 与依赖架构

默认 feature 集仅包含核心能力。`filter` 提供 filter 所需依赖，`schema` 包含 `filter`，`json`
提供有界 JSON 支持及 JSON budget 集成；各类值类型独立由 feature 控制。应用应按边界只启用实际
需要的能力层。

## 兼容策略

本版本保持公开名称和 V1 wire 表示稳定。已存值的 schema 校验仍与兼容数值的 filter 检查分离。
集成方应使用 builder 和公开 Serde API，让 wire 数据保持在规范限制内，并把未来 wire 版本变化
作为明确的兼容事件处理。
