<!-- Copyright The odrive-can-protocol Contributors -->
<!-- 本文件说明框架适配模块、依赖版本和纯软件验证方式。 -->

# 示例

这里的示例只演示 `odrive-can-protocol` 的协议值与帧表示之间的转换；不连接设备、不发送 CAN，
也不包含任务、时序或控制流程。

## 独立编码与解码

[`encode_decode.rs`](encode_decode.rs) 使用独立给定的协议字节，演示命令编码、RTR 查询与
Heartbeat 解码：

```sh
cargo run --example encode_decode
```

## 框架兼容矩阵

适配模块随 crates.io 包分发，复制到应用中使用；核心 crate 继续保持零依赖、零 feature。
各模块只转换帧，不依赖执行器，也不创建 CAN 外设、socket、任务或重试循环。

| 项目／生态 | 已核对版本与类型 | 适配入口 | 验证范围 |
|---|---|---|---|
| Embassy STM32 | `embassy-stm32 0.6.0`，`Frame`／`FdFrame` | [embassy/adapter.rs](embassy/adapter.rs) | 原生帧测试；H723 FDCAN、F405 bxCAN 交叉编译 |
| embedded-can／bxCAN | `embedded-can 0.4.1`、`bxcan 0.8.0` | [embedded_can/adapter.rs](embedded_can/adapter.rs) | bxCAN 原生帧测试、Cortex-M 交叉编译 |
| RTIC／STM32 HAL | 应用驱动使用上述 `bxcan::Frame` 或兼容的 Classic `embedded_can::Frame` | 同上 | 复用驱动帧转换；执行器和具体 HAL 应用由消费方验证 |
| Linux SocketCAN／Tokio | `socketcan 4.0.0`，`CanFrame`／`CanAnyFrame` | [socketcan/adapter.rs](socketcan/adapter.rs) | Linux 原生帧测试；同步与 Tokio 使用相同帧类型 |

这些版本是本次检查基线，不代表其他版本自动兼容。核心库 MSRV 仍为 1.85；示例消费方需满足
各依赖自己的 MSRV，例如 SocketCAN 4.0 要求 Rust 1.89。验证脚本使用 stable。

`embedded-can` 的 trait 不提供 FDF 或 Linux 错误帧标志，即使数据只有 8 字节也不能据此
认定为 Classic。Embassy 和 SocketCAN 必须使用专用模块保留这些语义。

## Embassy STM32

[`embassy/adapter.rs`](embassy/adapter.rs) 是可复制到消费方应用的 Rust 模块，不是本 crate 的
独立 Cargo example。将该文件复制到应用的 `src/` 目录后，应用可按自己的模块布局引用它；
例如放为 `src/odrive_can_protocol_adapter.rs`：

```rust
#[path = "odrive_can_protocol_adapter.rs"]
mod odrive_can_protocol_adapter;
```

应用的 `Cargo.toml` 需要自行声明 HAL 依赖。以下为 STM32H723 的示例，
`stm32h723vg` 只选择消费方的芯片；其他芯片应替换为应用实际需要的 Embassy feature：

```toml
[dependencies]
odrive-can-protocol = "0.1.1"
embassy-stm32 = { version = "=0.6.0", features = ["stm32h723vg"] }
embedded-can = "=0.4.1"
```

模块公开 `to_embassy` 和 `from_embassy`：前者把 `EncodedFrame` 转为
`embassy_stm32::can::Frame`，后者把 `Frame` 借用为 `FrameRef`。它们只做数据转换：Classic
数据帧按头部长度提供有效字节，RTR 保留 DLC，带 FDF 标志的帧即使缓冲不超过 8 字节也保留为
CAN FD，供协议层显式拒绝。使用 `FdFrame` 容器的应用可调用 `to_embassy_fd`／
`from_embassy_fd`：发送时仍构造 FDF=false 的 Classic 帧，接收时按真实 FDF 标志区分。

## embedded-can、bxCAN 与 RTIC

复制 [`embedded_can/adapter.rs`](embedded_can/adapter.rs) 到应用 `src/adapter.rs`：

```toml
[dependencies]
odrive-can-protocol = "0.1.1"
embedded-can = "=0.4.1"
bxcan = "=0.8.0"
```

```rust
mod adapter;
use odrive_can_protocol::{Message, NodeId, Query, encode};

let encoded = encode(NodeId::new(1).unwrap(), Message::Request(Query::EncoderEstimates)).unwrap();
let native: bxcan::Frame = adapter::to_embedded_can(&encoded).unwrap();
let view = adapter::from_embedded_can(&native).unwrap();
```

RTIC 应用和 STM32 HAL 可使用同一模块；由应用持有 CAN 外设与中断。通用函数要求原生帧
只表示 Classic 数据或 RTR，不能用于会把 FD／错误帧隐藏在 trait 后面的混合类型。

`fdcan 0.2.1` 暂未列为发送兼容：其 `TxFrameHeader` 用 `len == 0` 设置 RTR，不能同时
表达本库 DLC 0 数据命令和 DLC 8 RTR 查询；不能靠修改 DLC 来绕过。
依据见 [fdcan 0.2.1 帧实现](https://docs.rs/crate/fdcan/0.2.1/source/src/frame.rs)。

## Linux SocketCAN 与 Tokio

复制 [`socketcan/adapter.rs`](socketcan/adapter.rs) 到应用 `src/adapter.rs`：

```toml
[dependencies]
odrive-can-protocol = "0.1.1"
embedded-can = "=0.4.1"
socketcan = { version = "=4.0.0", default-features = false }
```

`to_socketcan` 返回 Classic `CanFrame`。`from_socketcan` 接受 `CanAnyFrame`，保留 FD
标志并以 `FromSocketcanError::ErrorFrame` 拒绝 Linux 错误通知。使用 Classic 接口得到
`CanFrame` 时，先调用 `let frame = socketcan::CanAnyFrame::from(frame);` 再借用。
Tokio 应用可在同一依赖上启用 `features = ["tokio"]`，复用相同转换函数。

## 自动验证

[`check.py`](check.py) 创建临时消费方清单，依赖本地协议库并直接引用待发布的适配源码。
不会修改根 Cargo.toml／Cargo.lock；首次运行会下载对应驱动依赖。
原生帧测试使用独立给定的协议字节，覆盖有效长度、扩展 ID、RTR、短 FD 与错误通知。
CI 执行以下检查（SocketCAN 要求 Linux）：

```sh
python3 examples/check.py embedded_can
python3 examples/check.py embassy
python3 examples/check.py socketcan
python3 examples/check.py embedded_can --target thumbv7em-none-eabihf
python3 examples/check.py embassy --target thumbv7em-none-eabihf --chip stm32h723vg
python3 examples/check.py embassy --target thumbv7em-none-eabihf --chip stm32f405rg
```

无 `--target` 时检查格式、运行测试与 Clippy；交叉检查验证 `no_std` 构建，不执行实板程序。
根目录 `cargo test --all-targets` 只覆盖协议库与独立示例，不能代替上述适配检查。

API 依据：[Embassy 0.6.0 帧源码](https://docs.rs/crate/embassy-stm32/0.6.0/source/src/can/frame.rs)、
[embedded-can 0.4.1](https://docs.rs/embedded-can/0.4.1/embedded_can/trait.Frame.html)、
[bxCAN 0.8.0](https://docs.rs/bxcan/0.8.0/bxcan/struct.Frame.html)、
[SocketCAN 4.0.0 帧源码](https://docs.rs/crate/socketcan/4.0.0/source/src/frame.rs)。
