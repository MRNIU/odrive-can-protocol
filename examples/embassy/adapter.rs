// Copyright The odrive-can Contributors

//! Embassy STM32 Classic CAN 帧与 `odrive_can_protocol` 帧视图之间的纯转换。
//!
//! 本模块不持有外设，也不发送或接收 CAN。`to_embassy` 只将
//! [`odrive_can_protocol::EncodedFrame`] 转为 Classic CAN `Frame`；`from_embassy`
//! 借用输入帧，并对 Classic 数据帧只暴露 `Header::len()` 指定的有效字节。
//! RTR 帧保留其 DLC 而不读取数据缓冲。FDF 标志优先于 RTR：即使缓冲不超过 8 字节，
//! FDF 帧也会映射为 `FramePayload::Fd`，由协议层拒绝而不会被误作 Classic CAN。

use embassy_stm32::can::{Frame, enums::FrameCreateError};
use embedded_can::{Id, StandardId};
use odrive_can_protocol::{EncodedFrame, FrameId, FramePayload, FrameRef};

/// 把协议编码器产生的 Classic 标准 CAN 帧转换为 Embassy 帧。
///
/// `EncodedFrame` 已保证其标识符和 DLC 适用于 Classic CAN；本函数不发送该帧。
pub fn to_embassy(frame: &EncodedFrame) -> Result<Frame, FrameCreateError> {
    let id = StandardId::new(frame.id()).ok_or(FrameCreateError::InvalidCanId)?;
    if frame.is_remote() {
        Frame::new_remote(id, usize::from(frame.dlc()))
    } else {
        Frame::new_data(id, frame.data())
    }
}

/// 从 Embassy Classic `Frame` 创建借用帧视图时的错误。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FromEmbassyError {
    /// `Header::len()` 超出 `Frame::raw_data()` 可提供的字节范围。
    InvalidPayloadLength,
}

/// 把 Embassy `Frame` 借用为协议解码器需要的帧视图。
///
/// 本函数不接收 CAN，也不验证 ODrive 协议。对 FDF 帧保留其标志；对 RTR 帧保留 DLC；
/// 对 Classic 数据帧仅返回 `Header::len()` 指定的有效字节。
pub fn from_embassy(frame: &Frame) -> Result<FrameRef<'_>, FromEmbassyError> {
    let id = match *frame.id() {
        Id::Standard(id) => FrameId::Standard(id.as_raw()),
        Id::Extended(id) => FrameId::Extended(id.as_raw()),
    };
    let header = frame.header();
    let payload = if header.fdcan() {
        FramePayload::Fd(frame.raw_data())
    } else if header.rtr() {
        FramePayload::Remote { dlc: header.len() }
    } else {
        let bytes = frame
            .raw_data()
            .get(..usize::from(header.len()))
            .ok_or(FromEmbassyError::InvalidPayloadLength)?;
        FramePayload::Data(bytes)
    };
    Ok(FrameRef { id, payload })
}
