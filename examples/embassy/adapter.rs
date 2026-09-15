// Copyright The odrive-can-protocol Contributors

//! Embassy STM32 Classic CAN 帧与 `odrive_can_protocol` 帧视图之间的纯转换。
//!
//! 本模块不持有外设，也不发送或接收 CAN。`to_embassy` 只将
//! [`odrive_can_protocol::EncodedFrame`] 转为 Classic CAN `Frame`；`from_embassy`
//! 借用输入帧，并对 Classic 数据帧只暴露 `Header::len()` 指定的有效字节。
//! RTR 帧保留其 DLC 而不读取数据缓冲。FDF 标志优先于 RTR：即使缓冲不超过 8 字节，
//! FDF 帧也会映射为 `FramePayload::Fd`，由协议层拒绝而不会被误作 Classic CAN。

use embassy_stm32::can::{
    Frame,
    enums::FrameCreateError,
    frame::{FdFrame, Header},
};
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

/// 将编码结果放入 Embassy `FdFrame` 容器，头部仍为 Classic CAN（FDF 与 BRS 均关闭）。
///
/// 适用于使用 FD 接收／发送接口但发送 Classic CANSimple 的应用；RTR 原样保留 DLC。
/// 本函数不启用外设 CAN FD 模式，也不发送帧。
pub fn to_embassy_fd(frame: &EncodedFrame) -> Result<FdFrame, FrameCreateError> {
    let id = StandardId::new(frame.id()).ok_or(FrameCreateError::InvalidCanId)?;
    FdFrame::new(
        Header::new(id.into(), frame.dlc(), frame.is_remote()),
        frame.data(),
    )
}

/// 从 Embassy 帧创建借用帧视图时的错误。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FromEmbassyError {
    /// `Header::len()` 超出帧缓冲可提供的字节范围。
    InvalidPayloadLength,
}

/// 把 Embassy `Frame` 借用为协议解码器需要的帧视图。
///
/// 本函数不接收 CAN，也不验证 ODrive 协议。对 FDF 帧保留其标志；对 RTR 帧保留 DLC；
/// 对 Classic 数据帧仅返回 `Header::len()` 指定的有效字节。
pub fn from_embassy(frame: &Frame) -> Result<FrameRef<'_>, FromEmbassyError> {
    from_parts(frame.header(), frame.raw_data())
}

/// 借用 Embassy `FdFrame` 容器，按头部 FDF 标志区分 Classic 与 CAN FD。
///
/// 先检查长度再访问 `FdFrame::data()`，避免公开构造器产生的超长头部导致切片 panic。
/// CAN FD 保留标志供协议层拒绝；Classic RTR 不读取数据并保留 DLC。
pub fn from_embassy_fd(frame: &FdFrame) -> Result<FrameRef<'_>, FromEmbassyError> {
    let header = frame.header();
    if !header.fdcan() && header.rtr() {
        return from_parts(header, &[]);
    }
    if header.len() > 64 {
        return Err(FromEmbassyError::InvalidPayloadLength);
    }
    from_parts(header, frame.data())
}

fn from_parts<'a>(header: &Header, data: &'a [u8]) -> Result<FrameRef<'a>, FromEmbassyError> {
    let id = match *header.id() {
        Id::Standard(id) => FrameId::Standard(id.as_raw()),
        Id::Extended(id) => FrameId::Extended(id.as_raw()),
    };
    let payload = if header.fdcan() {
        FramePayload::Fd(data)
    } else if header.rtr() {
        FramePayload::Remote { dlc: header.len() }
    } else {
        let bytes = data
            .get(..usize::from(header.len()))
            .ok_or(FromEmbassyError::InvalidPayloadLength)?;
        FramePayload::Data(bytes)
    };
    Ok(FrameRef { id, payload })
}
