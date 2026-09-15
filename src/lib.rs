// Copyright The odrive-can Contributors

//! ODrive CANSimple 协议库入口，提供版本明确的编解码 API 与硬件无关帧类型。
//!
//! 本库仅使用 core，不分配内存、不访问设备；协议转换结果与设备执行结果分别由调用方处理。

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

/// 与硬件无关的 CAN 帧表示。
pub mod frame;
/// ODrive 固件 `fw-v0.5.1` 的 CANSimple 协议。
pub mod fw_v0_5_1;

pub use frame::{EncodedFrame, FrameId, FramePayload, FrameRef};
pub use fw_v0_5_1::{
    AxisState, Command, DecodeError, EncodeError, Message, NodeId, Query, Response, decode, encode,
};
