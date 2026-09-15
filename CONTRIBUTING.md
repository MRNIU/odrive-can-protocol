<!-- Copyright The odrive-can Contributors -->
<!-- 本文件说明问题反馈、协议贡献、软件验证和版本维护流程。 -->

# 贡献指南

欢迎通过 [Issue](https://github.com/MRNIU/odrive-can/issues) 报告问题或讨论改进，
通过 [Pull Request](https://github.com/MRNIU/odrive-can/pulls) 提交变更。
小型修复可以直接提交 PR；新增固件支持或改变公共 API 时，建议先在 Issue 中说明使用需求和兼容性影响。

## 报告问题

请提供能够复现问题的最小代码、crate 与 Rust 版本、预期结果和实际结果。
协议问题还应附上固件版本或 Git revision，以及原始 CAN ID、帧类型、DLC 和有效载荷字节。
如果涉及厂家固件，请给出可核对的源码入口；仅有产品名称不足以确定协议布局。

## 开发约定

- 核心库保持 `no_std`、无动态分配、无外部依赖，不持有 CAN 外设或执行器。
- 库只负责协议值与帧的转换；任务、重试、时序、单位换算和控制策略由应用负责。
- 使用固定大小缓冲或借用切片，明确有效数据长度，保留未知状态值与错误位。
- Rust 标识符使用英文；说明文档与 rustdoc 使用中文，API 文档写明单位、前提和返回语义。
- 每个文件开头写明职责，并保留 `Copyright The odrive-can Contributors`，使用对应格式的合法注释。
- 保留 [MIT 许可证](LICENSE) 的许可条款与原版权信息。

## 协议变更与版本支持

项目支持范围见 [README](README.md#版本支持)。新增固件版本时，请在独立、明确命名的
协议模块中实现已核对的语义；多个版本可以共享经过确认一致的帧基础类型。
不要让已有版本模块随新固件改变含义，也不要通过帧长猜测固件版本。

每个协议变更应包含：

1. 对应版本的官方文档和实现链接，以及固定 Git revision。
2. 命令号、方向、帧类型、有效长度、端序、字段宽度和单位的交叉核对。
3. 文档与固件实现不一致时的明确说明；厂家兼容结论只覆盖实际核对的源码。
4. 独立给定的预期字节测试，以及受影响的非法输入或未知值测试。
5. README 支持矩阵、协议表格、rustdoc 和示例的必要更新。

测试应能捕获真实协议错误。仅做 encode/decode 自循环不能证明布局正确；也不需要为常量存在、
字段存在或依赖库已经保证的行为重复建立测试矩阵。

## 本地验证

安装 Rust stable，并添加交叉编译目标：

```sh
rustup component add rustfmt clippy
rustup target add thumbv7em-none-eabihf thumbv6m-none-eabi
```

在仓库根目录执行：

```sh
cargo fmt --check
cargo test --locked --all-targets
cargo test --locked --doc
cargo run --locked --example encode_decode
cargo clippy --locked --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
cargo build --locked --lib --target thumbv7em-none-eabihf
cargo build --locked --lib --target thumbv6m-none-eabi
cargo tree --locked --edges features
cargo metadata --locked --format-version 1
cargo package --locked
```

依赖图应只有本 crate，`dependencies` 为空，`features` 为空。当前 MSRV 为 Rust 1.85，
涉及语言特性或清单变更时还需验证：

```sh
rustup toolchain install 1.85.0 --profile minimal
cargo +1.85.0 check --locked --lib
```

未提交改动时，本地封包可使用 `cargo package --locked --allow-dirty`；发布前使用干净 checkout。
CI 执行格式、测试、文档、Clippy、两个 Cortex-M 目标构建、依赖检查、封包和 MSRV 检查。
具体驱动适配示例的验证方式见 [examples](examples/README.md)。这些软件检查不代表真实设备执行结果。

## 提交 Pull Request

PR 描述应说明具体问题、变更后的行为、兼容性影响，以及实际执行的验证命令和结果。
保持改动聚焦，避免混入无关格式调整。对公共 API 或协议行为的变更，请更新 [CHANGELOG](CHANGELOG.md)。

提交信息采用 Conventional Commits，例如 `fix: 保留 Heartbeat 状态高位`。
使用 `git commit --signoff` 添加 DCO 签署，表示符合 [Developer Certificate of Origin](https://developercertificate.org/)。
AI 协作提交应保留适用的 `Co-authored-by` 署名。

## 版本与发布

crate 版本与设备固件版本独立管理。维护者按 SemVer 发布；在 `0.x` 阶段，破坏兼容的 API
变更提升次版本，兼容修复提升补丁版本。提高 MSRV 时需在变更记录中说明。

发布前更新 Cargo 版本、lockfile 和变更记录，完成上述验证及 `cargo publish --locked --dry-run`。
维护者在配置好 crates.io 凭据的环境执行 `cargo publish --locked`，确认注册表中的版本、仓库链接
和文档构建结果后，再创建对应的 Git tag 与 GitHub Release。不要将发布凭据写入仓库。
