<!-- Copyright The odrive-can Contributors -->

# 示例

这里的示例只演示 `odrive-can-protocol` 的协议值与帧表示之间的转换；不连接设备、不发送 CAN，
也不包含任务、时序或控制流程。

## 独立编码与解码

[`encode_decode.rs`](encode_decode.rs) 使用独立给定的协议字节，演示命令编码、RTR 查询与
Heartbeat 解码：

```sh
cargo run --example encode_decode
```

## 在 Embassy STM32 应用中使用适配模块

[`embassy/adapter.rs`](embassy/adapter.rs) 是可复制到消费方应用的 Rust 模块，不是本 crate 的
独立 Cargo example。将该文件复制到应用的 `src/` 目录后，应用可按自己的模块布局引用它；
例如放为 `src/odrive_can_adapter.rs`：

```rust
#[path = "odrive_can_adapter.rs"]
mod odrive_can_adapter;
```

应用的 `Cargo.toml` 需要自行声明 HAL 依赖。以下为 STM32H723 的示例，
`stm32h723vg` 只选择消费方的芯片；其他芯片应替换为应用实际需要的 Embassy feature：

```toml
[dependencies]
odrive-can-protocol = "0.1"
embassy-stm32 = { version = "=0.6.0", features = ["stm32h723vg"] }
embedded-can = "=0.4.1"
```

模块公开 `to_embassy` 和 `from_embassy`：前者把 `EncodedFrame` 转为
`embassy_stm32::can::Frame`，后者把 `Frame` 借用为 `FrameRef`。它们只做数据转换：Classic
数据帧按头部长度提供有效字节，RTR 保留 DLC，带 FDF 标志的帧即使缓冲不超过 8 字节也保留为
CAN FD，供协议层显式拒绝。

本模块不属于核心 crate 的构建图，因此 `cargo test --all-targets` 不会自动编译它；核心
`Cargo.toml` 也不会为此加入 HAL 依赖。复制到已有 Embassy 应用后，可在该应用目录验证：

```sh
cargo check --lib --target thumbv7em-none-eabihf
```
