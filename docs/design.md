<!-- Copyright The odrive-can Contributors -->

# 首版设计

本文件说明通用协议库的数据边界、API 取舍和源码组织。

## 目标与边界

单个 `odrive-can` library crate，Rust stable、2024 Edition，`#![no_std]`、禁止 unsafe，
零依赖、无 `alloc`。唯一协议模块为 `fw_v0_5_1`。保留 MIT 原许可条款及版权。
版本依据和文档／实现差异由 [protocol.md](protocol.md) 记录。

采用固定大小编码产物和借用的输入帧视图：比绑定 HAL 更适合主机和不同 MCU；当前没有
引入 `embedded-can` 的必要。帧视图显式区分标准／扩展 ID、Classic 数据／RTR／CAN FD。
首版编码仅标准 Classic CAN；不提前建立其他协议版本或扩展插件。

## API 与结果

- `NodeId::new(u32)` 验证 `0..=63`，不掩码截断；节点 63 在此版本没有新版广播语义。
- `Command` 表达主机写命令，`Query` 表达 RTR 读取，`Response` 表达设备数据；
  `Message` 合并三种方向，支持离线分析及双方编解码。
- `AxisState(u32)` 保留完整状态，已知状态仅为便利常量；所有错误掩码保留 `u32`。
- `encode(node, message)` 产出 ID、DLC、有效数据和 RTR 标志。命令的浮点输入必须有限，
  接收端浮点数据如实保留；不进行物理范围裁决。位置命令的两个 `i16` 前馈直接表达
  协议的千分之一单位，避免引入隐式舍入／饱和策略。
- `decode(node, FrameRef)` 返回 `Result<Option<Message>, DecodeError>`。先排除不匹配的
  标准节点／未知命令，再检查相关消息形态及长度；扩展 ID 明确返回不支持，相关 CAN FD
  或错误 RTR 形态返回错误。`0x08` 在官方未实现，明确返回不支持；不伪造有效命令。
- 写命令使用恰好容纳定义字段的 0／4／8 字节；设备回复按源码统一 DLC 8，尾部未定义
  字节解码时忽略，编码时清零。RTR 编码 DLC 8，解码接受 0..=8，因为源码不检查请求 DLC。
- Set Axis Requested State 按文档编码完整 u32；明确说明官方实现只读取低 16 位。

成功仅表示协议转换完成；编码、CAN 发送、设备状态与运动结果是独立事实。库不持有
外设、任务、队列、计时、重试、心跳新鲜度、单位换算、限幅、许可、停止或恢复策略。

## 文件组织与验证

1. `src/lib.rs`、`src/frame.rs`、`src/fw_v0_5_1.rs`：公共边界、帧类型、完整协议编解码。
2. `tests/protocol.rs`：独立字节向量覆盖命令、遥测、RTR、未知值和边界错误。
3. `README.md`、`docs/protocol.md`：通用使用说明、交接时序和固定 revision 协议证据。
4. `Cargo.toml`、`.gitignore`、`.github/workflows/ci.yml`、`AGENTS.md`：最小工程配置。
5. `examples/`：可运行的协议转换示例，以及供应用选用的驱动帧适配模块。

验证包括 fmt、全部 targets 测试、Clippy、rustdoc（含 doctest）、两个 Cortex-M target
build、依赖／feature 核对和本地 package。实际结果见 [验证记录](validation.md)。
