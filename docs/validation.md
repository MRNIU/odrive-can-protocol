<!-- Copyright The odrive-can Contributors -->

# 软件验证记录

本文件记录协议库的检查命令、覆盖范围和软件证据限制，便于维护者复现验证。

核对日期：2026-09-15。工具链为 Rust／Cargo 1.98.1，另验证最低版本 Rust 1.85.0。

## 已执行检查

下表命令在仓库目录执行，全部退出码为 0。

| 命令 | 结果 |
|---|---|
| `cargo fmt --check` | 通过 |
| `cargo test --locked --all-targets` | 9 个协议测试通过；库和 example target 编译通过 |
| `cargo test --locked --doc` | README 核心示例通过 |
| `cargo run --locked --example encode_decode` | ID `0x02d`，字节 `00 00 20 c0 00 00 80 3e`；完整解出状态 `65544` |
| `cargo clippy --locked --all-targets -- -D warnings` | 通过，无警告 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps` | 通过 |
| `cargo build --locked --lib --target thumbv7em-none-eabihf` | 通过 |
| `cargo build --locked --lib --target thumbv6m-none-eabi` | 通过 |
| `cargo +1.85.0 check --locked --lib` | 通过 |
| `cargo tree --locked --edges features` | 只有 `odrive-can v0.1.0` |
| `cargo metadata --locked --format-version 1` | 1 个 package；dependencies `[]`，features `{}` |
| `cargo package --locked --allow-dirty` | 本地封包和解包后的重新构建通过，未上传 |
| `cargo package --locked --allow-dirty --list` | 包含源码、测试、示例、文档及 MIT 许可，不包含编译产物 |

`--allow-dirty` 用于有未提交改动时的本地验证，干净 checkout 可省略此参数。
CI 配置覆盖主检查和 MSRV；本记录是本地执行结果，不代替远端 CI run。

## 测试与依赖边界

9 个表驱动测试使用独立给定的 ID／字节向量检查 14 个写命令、8 个 RTR 查询和
9 类设备数据，不只依赖 encode/decode 自循环。覆盖：

- 正／负／正零／负零速度、非零转矩前馈及主机浮点字段 NaN／正负无穷。
- 非法节点、标准 ID 越界、相关长度错误、Heartbeat RTR、FD／扩展 ID、未实现命令。
- 无关节点／未知命令、完整 u32 状态／错误高位、设备浮点 NaN 及 error 回复尾部处理。

库根使用 `#![no_std]`，禁止 unsafe；无外部依赖、`alloc` 或 feature 开关。
主机测试与 example 使用 `std` 不改变库的依赖合同。具体驱动的适配示例独立验证，
方法见 [使用示例](../examples/README.md)，其依赖不进入协议库。

## 协议复核与证据范围

独立复核已按固定官方源码核对命令号、方向、端序、字段、DLC、单位及未知值保留，
并检查输入边界、依赖和返回语义，未发现遗留协议实现问题。

这些检查证明协议转换和软件可用性，不证明真实总线收发、设备执行或运动结果。
官方及 MKS revision、文件 SHA-256 和已知固件差异见 [协议证据](protocol.md)。
