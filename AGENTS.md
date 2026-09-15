<!-- Copyright The odrive-can Contributors -->

# 本仓库开发约束

本文件约定协议实现、文档和验证的维护边界，适用于本仓库全部源码与示例。

- 保持单个 library crate、Rust stable／2024 Edition、`no_std`、无 `alloc`、无硬件依赖。
- 首版语义固定为 ODrive `fw-v0.5.1`；协议变更必须核对版本源码，并更新 `docs/protocol.md`。
- 未知状态和错误位是有效协议数据，不可截断或变成解码错误。
- 不添加外设访问、任务、重试、时序、单位换算、限幅或业务许可／恢复逻辑。
- 文档使用中文；精确 API 前提、单位和返回语义放在 rustdoc。
- 测试使用独立给定的协议字节；变更后运行 README 中相应软件检查。
- 文件开头保留 `Copyright The odrive-can Contributors`，并说明文件职责；注释采用各格式的合法语法。
- 保留 MIT LICENSE 的原许可条款和版权信息。
- 通用文档不包含本地开发环境或特定消费方描述；具体框架的接入方法放在 `examples/`。
