<!-- Copyright The odrive-can-protocol Contributors -->
<!-- 本文件说明唯一示例与可选兼容层的消费方接入。 -->

# 示例与兼容层

仓库仅保留 [`encode_decode.rs`](encode_decode.rs)，它以独立给定的协议字节演示编码、RTR 查询和 Heartbeat 解码：

```sh
cargo run --example encode_decode
```

可选兼容层由 crate 本身提供。默认构建仍为 `no_std`、无 `alloc`、零依赖；启用 Embassy STM32 或 SocketCAN 时会自动启用 `embedded-can`。

下列片段中的 `encoded` 是 `encode()` 返回的帧；取得 `view` 后可直接调用 `decode(node, view)`。

## `embedded-can`

```toml
[dependencies]
odrive-can-protocol = { version = "0.1.1", features = ["embedded-can"] }
embedded-can = "0.4"
bxcan = "0.8"
```

```rust
use odrive_can_protocol::FrameRef;

let native: bxcan::Frame = encoded.to_embedded_can().unwrap();
let view = FrameRef::from_classic_embedded_can(&native)?;
```

此入口只接收已确认的 Classic 数据帧或 RTR。通用 trait 不携带 CAN FD 或 Linux 错误帧语义；`InvalidFrameLength` 位于 `odrive_can_protocol::compat::embedded_can`。

## Embassy STM32

```toml
[dependencies]
odrive-can-protocol = { version = "0.1.1", features = ["embassy-stm32"] }
embassy-stm32 = { version = "0.6", features = ["stm32h723vg"] }
```

`embassy-stm32` 的芯片 feature 由消费方选择；本 crate 不注入芯片型号。`embassy_stm32::can::Frame` 和 `FdFrame` 可直接从 `&EncodedFrame` 构造，接收帧可通过 `FrameRef::try_from(&frame)` 借用：

```rust
use embassy_stm32::can::Frame;
use odrive_can_protocol::FrameRef;

let native = Frame::from(&encoded);
let view = FrameRef::try_from(&native)?;
```

转换错误位于 `odrive_can_protocol::compat::embassy`。`FdFrame` 入口保留实际帧形态，协议层会拒绝 CAN FD。

## Linux SocketCAN

```toml
[dependencies]
odrive-can-protocol = { version = "0.1.1", features = ["socketcan"] }
socketcan = "4"
```

此 feature 仅支持 Linux，SocketCAN 4 需要 Rust 1.89。`socketcan::CanFrame` 可从 `&EncodedFrame` 直接构造；`FrameRef::try_from(&frame)` 接受 `CanFrame` 与 `CanAnyFrame`：

```rust
use odrive_can_protocol::FrameRef;
use socketcan::CanFrame;

let native = CanFrame::from(&encoded);
let view = FrameRef::try_from(&native)?;
```

转换错误位于 `odrive_can_protocol::compat::socketcan`；它保留 CAN FD 标志并拒绝 Linux 错误通知。Tokio 接口复用相同帧类型，启用驱动的 `tokio` feature 即可。

## 检查

```sh
cargo test --locked --all-targets --all-features --features embassy-stm32/stm32h723vg
cargo clippy --locked --all-targets --all-features --features embassy-stm32/stm32h723vg -- -D warnings
cargo check --locked --lib --target thumbv7em-none-eabihf --features embassy-stm32,embassy-stm32/stm32h723vg
cargo check --locked --lib --target thumbv7em-none-eabihf --features embassy-stm32,embassy-stm32/stm32f405rg
cargo +1.85.0 check --locked --lib --features embedded-can
```

启用可选依赖时，应在目标消费方的 Rust、操作系统和芯片配置下构建；可选依赖的 MSRV 由各自项目定义。
