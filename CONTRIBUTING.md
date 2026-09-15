<!-- Copyright The odrive-can-protocol Contributors -->
<!-- 本文件说明协议贡献边界、验证和发布流程。 -->

# 贡献指南

欢迎提交 [Issue](https://github.com/MRNIU/odrive-can-protocol/issues) 和 [Pull Request](https://github.com/MRNIU/odrive-can-protocol/pulls)。请提供可复现的最小代码、crate 与 Rust 版本；协议问题还应附固件 revision、CAN ID、帧类型、DLC 与有效载荷。

## 维护边界

- 默认库保持单个 `no_std`、无 `alloc`、无硬件依赖的 library crate。
- 协议层只转换协议值和帧；CAN 外设、任务、重试、时序、单位换算和控制策略由应用负责。
- 未知状态和错误位是协议数据，不能丢弃或改为解码错误。
- 文档与 rustdoc 使用中文；Rust 标识符使用英文，并写明 API 前提、单位和返回语义。
- 每个文件保留 `Copyright The odrive-can-protocol Contributors` 与职责说明，MIT 许可条款和版权信息不得删除。

## 协议与兼容层变更

新增固件版本必须使用固定 revision 的官方文档和实现交叉核对命令号、方向、帧形态、有效长度、端序、字段宽度与单位；为该版本提供明确模块、独立给定字节的测试，并更新 README 支持矩阵和源码依据。

兼容层应通过 optional feature 接入，不向默认构建引入依赖或硬件选择。若接入 Embassy STM32，芯片 feature 必须由消费方的 `embassy-stm32 0.6` 依赖选择。SocketCAN 4 仅支持 Linux，要求 Rust 1.89；默认与 `embedded-can` 支持 Rust 1.85，其它可选依赖遵循自己的 MSRV。

## 本地验证

在仓库根目录运行：

```sh
cargo fmt --check
cargo test --locked --all-targets
cargo test --locked --doc
cargo run --locked --example encode_decode
cargo clippy --locked --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
cargo build --locked --lib --target thumbv7em-none-eabihf
cargo build --locked --lib --target thumbv6m-none-eabi
cargo tree --locked --edges normal
cargo metadata --locked --format-version 1
cargo package --locked
cargo +1.85.0 check --locked --lib
```

可选 feature 的检查、依赖版本和目标环境见 [examples](examples/README.md)。这些软件检查不代表实板通信或设备动作结果。

## PR 与发布

PR 请说明问题、变更后的行为、兼容性影响与实际验证。提交使用 Conventional Commits，并用 `git commit --signoff` 添加 DCO；AI 协作保留适用的 `Co-authored-by`。

发布前完成上述验证及 `cargo publish --locked --dry-run`。维护者在已配置 crates.io 凭据的环境运行 `cargo publish --locked`，确认注册表、文档和仓库链接后再创建 tag 与 GitHub Release。
