// Copyright The odrive-can-protocol Contributors

//! `embedded-can` 0.4 的 Classic CAN 帧转换，可用于 bxCAN 0.8 等驱动。
//!
//! 调用方须确保输入类型只表示 Classic 数据帧或 RTR。此 trait 不暴露 FDF 或错误帧标志，
//! 因此 Embassy、SocketCAN 混合帧应使用各自的专用适配模块，不能仅凭长度判断 CAN FD。
//! 本模块不访问外设、不分配内存；驱动依赖由复制本模块的应用声明。

use embedded_can::{Frame, Id, StandardId};
use odrive_can_protocol::{EncodedFrame, FrameId, FramePayload, FrameRef};

/// 将编码结果转换为实现 `embedded_can::Frame` 的 Classic CAN 帧。
///
/// `F` 必须保证 `new` 构造 Classic 数据帧，`new_remote` 构造 RTR；例如 `bxcan::Frame`。
/// 返回 `None` 表示驱动构造器拒绝此帧；成功只表示转换完成，不表示已发送。
pub fn to_embedded_can<F: Frame>(frame: &EncodedFrame) -> Option<F> {
    let id = StandardId::new(frame.id())?;
    if frame.is_remote() {
        F::new_remote(id, usize::from(frame.dlc()))
    } else {
        F::new(id, frame.data())
    }
}

/// Classic CAN 帧的 DLC 或有效数据长度不符合驱动 trait 合同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidFrameLength;

/// 将已确认是 Classic CAN 的原生帧借用为协议视图。
///
/// 不识别 FDF 或总线错误；调用方必须在进入本函数前从原生类型中排除这些帧。
/// DLC 必须为 `0..=8`，数据帧缓冲至少包含 DLC 个字节；超出时返回错误。
/// 扩展 ID 原样保留，RTR 保留 DLC 且不读取数据，数据帧只借用有效前缀。
pub fn from_embedded_can<F: Frame>(frame: &F) -> Result<FrameRef<'_>, InvalidFrameLength> {
    let id = match frame.id() {
        Id::Standard(id) => FrameId::Standard(id.as_raw()),
        Id::Extended(id) => FrameId::Extended(id.as_raw()),
    };
    let dlc = frame.dlc();
    if dlc > 8 {
        return Err(InvalidFrameLength);
    }
    let payload = if frame.is_remote_frame() {
        FramePayload::Remote { dlc: dlc as u8 }
    } else {
        FramePayload::Data(frame.data().get(..dlc).ok_or(InvalidFrameLength)?)
    };
    Ok(FrameRef { id, payload })
}
