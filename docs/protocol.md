<!-- Copyright The odrive-can Contributors -->

# fw-v0.5.1 协议依据与兼容性

核对日期：2026-09-15。本文件记录版本证据；精确 API 输入与返回约定以 rustdoc 为准。

## 官方基线

仓库 `odriverobotics/ODrive` 的 tag `fw-v0.5.1` 对应 Git revision：
`7831d795235e5ef8535e4b46621a0721b458ec8f`。

实际交叉核对：

- [协议文档](https://github.com/odriverobotics/ODrive/blob/7831d795235e5ef8535e4b46621a0721b458ec8f/docs/can-protocol.md)
- [can_simple.cpp](https://github.com/odriverobotics/ODrive/blob/7831d795235e5ef8535e4b46621a0721b458ec8f/Firmware/communication/can_simple.cpp)
- [命令枚举 can_simple.hpp](https://github.com/odriverobotics/ODrive/blob/7831d795235e5ef8535e4b46621a0721b458ec8f/Firmware/communication/can_simple.hpp)
- [帧结构与字段读取 can_helpers.hpp](https://github.com/odriverobotics/ODrive/blob/7831d795235e5ef8535e4b46621a0721b458ec8f/Firmware/communication/can_helpers.hpp)
- [Classic CAN HAL 收发 interface_can.cpp](https://github.com/odriverobotics/ODrive/blob/7831d795235e5ef8535e4b46621a0721b458ec8f/Firmware/communication/interface_can.cpp)
- [Sensorless 单位](https://github.com/odriverobotics/ODrive/blob/7831d795235e5ef8535e4b46621a0721b458ec8f/Firmware/MotorControl/sensorless_estimator.hpp)

## 寻址与帧边界

标准 11-bit CAN ID = `(node_id << 5) | command_id`，节点 `0..=63`，命令占低 5 bit。
节点是轴的 CAN 地址，不是轴索引；同一块 ODrive 的两根轴可以有不同节点号。
节点 63 在本基线中是普通地址，没有后续版本的广播／发现语义。

厂家源码还允许配置扩展 ID（24-bit 节点），并比较 `msg.isExt`；本库首版只支持标准
Classic CAN，扩展 ID 显式报错。CAN FD 也不支持，即使其 payload 只有 8 字节仍报错。
输入视图必须保留真实帧形态与实际有效数据切片，不能把 FD 或 RTR 转成普通数据帧。

查询由 Master 发送 RTR，同 ID 的 Axis 数据帧表达回复。库编码查询 DLC 8、数据为空；
解码接受 RTR DLC `0..=8`，因为固件回调只检查 RTR，不检查请求 DLC。Heartbeat 没有 RTR
查询语义。数据帧中的回复也可能来自周期发送；协议没有请求序号或可据此匹配的 ACK。

## 完整消息表

`M` 表示 Master，`A` 表示 Axis。字段从 byte 0 开始顺序排列，多字节字段全部 little-endian；
`f32` 为 IEEE 754 binary32。表内长度是本库数据帧长度；所有 Axis 数据回复实际均为 DLC 8。
查询行另有 `M RTR → A data` 方向。共覆盖 **23 个有效命令号**、14 个写命令、8 个查询和
9 种设备数据（含 Heartbeat）。

| ID | 消息 | 方向 | 字段／单位 | 长度 |
|---|---|---|---|---:|
| `0x01` | Heartbeat | A data | axis error `u32`；axis state `u32` | 8 |
| `0x02` | Estop | M data | 无 | 0 |
| `0x03` | Get Motor Error | RTR → A | error `u32`；尾部 4 字节 | 8 |
| `0x04` | Get Encoder Error | RTR → A | error `u32`；尾部 4 字节 | 8 |
| `0x05` | Get Sensorless Error | RTR → A | error `u32`；尾部 4 字节 | 8 |
| `0x06` | Set Axis Node ID | M data | 新 node id `u32` | 4 |
| `0x07` | Set Axis Requested State | M data | requested state `u32` | 4 |
| `0x09` | Get Encoder Estimates | RTR → A | position `f32` turn；velocity `f32` turn/s | 8 |
| `0x0A` | Get Encoder Count | RTR → A | shadow count `i32`；count in CPR `i32` | 8 |
| `0x0B` | Set Controller Modes | M data | control mode `i32`；input mode `i32` | 8 |
| `0x0C` | Set Input Pos | M data | position `f32` turn；velocity FF `i16 × 0.001` turn/s；torque FF `i16 × 0.001` N·m | 8 |
| `0x0D` | Set Input Vel | M data | velocity `f32` turn/s；torque FF `f32` N·m | 8 |
| `0x0E` | Set Input Torque | M data | torque `f32` N·m | 4 |
| `0x0F` | Set Velocity Limit | M data | velocity limit `f32` turn/s | 4 |
| `0x10` | Start Anticogging | M data | 无 | 0 |
| `0x11` | Set Traj Vel Limit | M data | velocity limit `f32` turn/s | 4 |
| `0x12` | Set Traj Accel Limits | M data | acceleration `f32`；deceleration `f32`，均 turn/s² | 8 |
| `0x13` | Set Traj Inertia | M data | inertia `f32`，原始 controller 参数，协议文档未声明单位 | 4 |
| `0x14` | Get IQ | RTR → A | Iq setpoint `f32` A；Iq measured `f32` A | 8 |
| `0x15` | Get Sensorless Estimates | RTR → A | PLL position `f32` rad；velocity `f32` turn/s | 8 |
| `0x16` | Reboot ODrive | M data | 无 | 0 |
| `0x17` | Get Vbus Voltage | RTR → A | voltage `f32` V；尾部 4 字节 | 8 |
| `0x18` | Clear Errors | M data | 无 | 0 |

以下项目没有可实现的 fw-v0.5.1 消息语义：

- `0x08 Set Axis Startup Config`：文档和回调均未实现，库返回 `UnsupportedCommand`，不提供
  伪造的写命令或占位回调。
- `0x00` 与文档的 `0x700`：CANopen 保留项，不属于 CANSimple 功能。`0x700` 也不是 5-bit
  command；不能将其 OR 到节点 ID。在线标准 ID `0x700` 会解析为 node 56／command 0，
  返回无关消息。库不会屏蔽整个 `0x700..0x7ff` 地址段。
- `0x19..0x1f`：本版本没有对应 handler，返回无关消息。

## 文档与实现差异及本库决策

1. **Heartbeat 是两个完整 u32。** 未知状态保持原值，未知 error bits 原样保留。设备错误是
   有效协议数据；只有精确值 1 和 8 才分别等于 Idle 和 ClosedLoopControl。不能只读 byte 4。
2. **状态请求文档为 u32，固件只读取低 16 位。** `set_axis_requested_state_callback` 使用
   `can_getSignal<int32_t>(msg, 0, 16, true)`。本库按文档写完整 4 字节并保留未知值，但不能
   保证设备按完整 u32 执行请求。例如高位非零而低位为 1 的请求仍可能被固件当作 Idle；
   调用方必须选择该固件支持的状态。
3. **错误和 Vbus 回复不能按有效字段长度缩为 4 字节。** 源码显式 `len = 8`，后三个 error
   回复的尾部来自 `buf[8] = {0}`，Vbus 尾部显式清零。本库编码尾部清零，解码忽略尾部内容，
   并要求实际 DLC 8；既不把 MotorError 当 u64，也不把 Vbus 尾部当电流。
4. **严格接收长度是库合同。** 固件读取字段时并不校验 `msg.len`，部分写命令也不检查 RTR。
   本库写命令只接收表内恰好 0／4／8 字节和数据帧形态，不模仿固件对补齐短帧、额外字节或
   错误 RTR 的宽松处理。编码输出是协议字段所需长度，不能把这一策略描述成厂家拒绝行为。
5. **方向以源码为准。** 文档一些 Get 行标注 Master，但字段实际上是 Axis 回复；
   Get Vbus Voltage 虽然缺少查询星号，其回调也要求 `msg.rtr`。
6. **Sensorless 的位置是 rad。** 源码发送 `pll_pos_`，不能沿用 Encoder Estimates 的 turn。
7. **浮点与范围。** 编码主机命令拒绝全部非有限 `f32`，不实施正负、幅值或控制模式限制。
   解码保留线上浮点（包括非有限值），设备数据编码也允许这些值；应用负责数据质量判断。
   本库不会因 payload 长度或未知字段推测／切换固件版本。

## MKS ODrive Mini 兼容证据

实际核对厂家发布包：**ODriveMINI-fw-v0.5.1-20250326**，位于
[`makerbase-motor/MKS-ODrive` revision `e15782976ae93d42b1f0648ceec96503141a343b`](https://github.com/makerbase-motor/MKS-ODrive/blob/e15782976ae93d42b1f0648ceec96503141a343b/Firmware/MKS%20ODrive%20MINI/ODriveMINI-fw-v0.5.1-20250326.rar)。

RAR SHA-256：`a8cbc2af68deccdadad150e20d8337cc7a1ae430f8ccb441625315e2f04d2430`。
解压后 `docs/can-protocol.md`、`Firmware/communication/can_simple.cpp` 和
`Firmware/communication/can_helpers.hpp` 与上述官方 revision 的同路径文件逐字节一致。

三个一致文件的 SHA-256（用于复查发布包）：

| 文件 | SHA-256 |
|---|---|
| `docs/can-protocol.md` | `1d6dcd3798df071313a667bd2a076949d73fe40cb2c18c0f7644faaf6b658fb5` |
| `Firmware/communication/can_simple.cpp` | `b2d0a8f78605fe3bc0e411bd3649bbf562d0db4eec39fd1dccc98998436741bd` |
| `Firmware/communication/can_helpers.hpp` | `b171f27d478493a6ea714a20cd86bd8d625312f886f5a0f671044c4e38169c4b` |

重点三类消息的源码依据：

| 消息 | MKS 源码依据 | 结论 |
|---|---|---|
| Heartbeat | `send_heartbeat`，DLC 8，error/state 分别写入 4 字节 | 两个完整 little-endian u32 一致 |
| Set Axis Requested State | `set_axis_requested_state_callback`，读取低 16 bit | 规范 u32 字段一致；保留官方实现限制 |
| Set Input Vel | `set_input_vel_callback`，offset 0/32 bit 读取两个 float | turn/s 速度与 N·m 转矩前馈一致 |

结论限定于这个**发布源码包**，并且首版只使用其标准 Classic CAN 子集。未操作设备、
未确认任何具体设备实际刷入的二进制，也没有进行总线或运动验证。源码相同不等于其他 MKS
版本、厂商修改版或以后版本都兼容；库不根据 USB 版本字符串或帧长度自动识别它们。

## 参考项目的适用边界

只读参考 [`raoz/odrive-messages`](https://github.com/raoz/odrive-messages/tree/37990cb157f667cdd0ddb441f907066e4126fbef)，
实际核对 Git revision `37990cb157f667cdd0ddb441f907066e4126fbef`。其当前消息表属于更新版：
`0x00` 是 GetVersion、`0x03` 是复合错误、`0x04/0x05` 是 SDO、`0x15` 是温度、`0x17`
含母线电流，Heartbeat 布局也不同。因此仅参考纯协议层的职责分离，不使用其 wire 格式
替代本基线，不复制其代码或引入其依赖。
