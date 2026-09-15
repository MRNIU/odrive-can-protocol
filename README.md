<!-- Copyright The odrive-can-protocol Contributors -->
<!-- 本文件介绍项目用途、支持范围、可选兼容层与使用入口。 -->

# odrive-can-protocol

[![CI](https://github.com/MRNIU/odrive-can-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/MRNIU/odrive-can-protocol/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/odrive-can-protocol.svg)](https://crates.io/crates/odrive-can-protocol)
[![docs.rs](https://docs.rs/odrive-can-protocol/badge.svg)](https://docs.rs/odrive-can-protocol)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/MRNIU/odrive-can-protocol/blob/main/LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)

`odrive-can-protocol` 是 ODrive CANSimple 的硬件无关编解码库。默认构建使用 Rust 2024、`#![no_std]`、无 `alloc`、零依赖，MSRV 为 Rust 1.85；它不访问 CAN 外设，也不管理任务、时序或控制策略。

## 安装

```toml
[dependencies]
odrive-can-protocol = "0.1.1"
```

Rust 中的 crate 名为 `odrive_can_protocol`。完整 API 见 [rustdoc](https://docs.rs/odrive-can-protocol)。

## 快速开始

```rust
use odrive_can_protocol::fw_v0_5_1::{encode, Command, Message, NodeId};

let frame = encode(
    NodeId::new(1).unwrap(),
    Message::Command(Command::SetInputVel {
        velocity: -2.5,
        torque_ff: 0.25,
    }),
)
.unwrap();

assert_eq!(frame.id(), 0x02d);
```

更多编码、RTR 查询和 Heartbeat 解码见 [`examples/encode_decode.rs`](https://github.com/MRNIU/odrive-can-protocol/blob/main/examples/encode_decode.rs)：

```sh
cargo run --example encode_decode
```

## 协议支持

当前 API 位于 `odrive_can_protocol::fw_v0_5_1`，crate 根同时重导出主要类型与 `encode`、`decode`。它支持标准 11-bit Classic CAN；多字节字段为 little-endian，浮点字段为 IEEE 754 binary32。

| 固件 | 模块 | 状态 |
|---|---|---|
| ODrive `fw-v0.5.1` | `fw_v0_5_1` | 已支持 |
| MKS ODrive Mini `ODriveMINI-fw-v0.5.1-20250326` | `fw_v0_5_1` | 已支持，同一 CANSimple 布局 |

MKS 结论仅覆盖该固定源码包：[ODriveMINI-fw-v0.5.1-20250326.rar](https://github.com/makerbase-motor/MKS-ODrive/blob/e15782976ae93d42b1f0648ceec96503141a343b/Firmware/MKS%20ODrive%20MINI/ODriveMINI-fw-v0.5.1-20250326.rar)。该包的 CANSimple 源码与官方 [ODrive `fw-v0.5.1`](https://github.com/odriverobotics/ODrive/tree/7831d795235e5ef8535e4b46621a0721b458ec8f) 基线一致，因此无需 MKS 专用 feature 或转换层。其他 MKS 固件版本应先核对源码。

## 可选兼容层

兼容层是 crate 的 optional feature；启用后可直接使用原生帧类型，无需自行复制转换代码。`embedded-can` 是 Embassy STM32 与 SocketCAN 的共同基础，后两者会自动启用它。

| feature | 提供的接入 | 前提 |
|---|---|---|
| `embedded-can` | 通用 Classic `embedded_can::Frame` 转换 | MSRV 1.85 |
| `embassy-stm32` | `embassy_stm32::can::Frame`、`FdFrame` 转换 | 消费方选择芯片与 `embassy-stm32 0.6` feature |
| `socketcan` | Linux `socketcan::CanFrame`、`CanAnyFrame` 转换 | Linux、SocketCAN 4、Rust 1.89 |

`embedded-can` 使用 `EncodedFrame::to_embedded_can::<F>() -> Option<F>` 创建原生 Classic 帧；`FrameRef::from_classic_embedded_can(&frame)` 借用解码，并在 `compat::embedded_can::InvalidFrameLength` 中报告无法表示的长度。该 trait 不携带 CAN FD 或 Linux 错误帧语义，因此该入口只接收已确认的 Classic 帧。

启用 `embassy-stm32` 后，`embassy_stm32::can::Frame` 与 `FdFrame` 均实现 `From<&EncodedFrame>`，`FrameRef` 实现对二者借用的 `TryFrom`；错误类型位于 `compat::embassy`。启用 `socketcan` 后，`socketcan::CanFrame` 实现 `From<&EncodedFrame>`，`FrameRef` 实现对 `CanFrame` 和 `CanAnyFrame` 借用的 `TryFrom`；错误类型位于 `compat::socketcan`。

驱动依赖、芯片选择和最小接入示例见 [examples](https://github.com/MRNIU/odrive-can-protocol/blob/main/examples/README.md)。

## API 边界

`encode(node, message)` 只完成协议编码；`decode(node, frame)` 只还原该节点的已知消息。设备错误位、未知状态和线上浮点位模式作为协议数据保留。应用负责收发、设备许可、超时、重试、单位换算和动作结果判断。

## 贡献与验证

```sh
cargo fmt --check
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo +1.85.0 check --lib
```

协议变更须使用固定版本的官方文档和实现核对，并更新此支持矩阵。详细要求见 [CONTRIBUTING.md](https://github.com/MRNIU/odrive-can-protocol/blob/main/CONTRIBUTING.md)。

## License

本项目采用 [MIT License](https://github.com/MRNIU/odrive-can-protocol/blob/main/LICENSE)，保留原版权信息。
