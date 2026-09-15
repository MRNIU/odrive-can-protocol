// Copyright The odrive-can-protocol Contributors

//! Linux SocketCAN 4.0 帧与协议视图的纯转换，适用于同步和 Tokio 接口返回的帧。
//!
//! 不打开 socket；保留扩展 ID、RTR DLC 和 CAN FD 类型，显式拒绝 Linux 错误帧。
//! 应用须自行声明 `socketcan` 与 `embedded-can` 依赖。

use embedded_can::{Frame, Id, StandardId};
use odrive_can_protocol::{EncodedFrame, FrameId, FramePayload, FrameRef};
use socketcan::{CanAnyFrame, CanFrame};

/// 将协议编码结果转换为可交给 SocketCAN 的 Classic 帧。
///
/// 返回 `None` 表示原生构造器拒绝此帧；不打开 socket 或发送数据。
pub fn to_socketcan(frame: &EncodedFrame) -> Option<CanFrame> {
    let id = StandardId::new(frame.id())?;
    if frame.is_remote() {
        CanFrame::new_remote(id, usize::from(frame.dlc()))
    } else {
        CanFrame::new(id, frame.data())
    }
}

/// SocketCAN 帧不能表示为 ODrive 输入视图的原因。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FromSocketcanError {
    /// Linux 总线错误通知，不是 ODrive 协议帧；应用应独立处理原生错误。
    ErrorFrame,
}

/// 将原生混合帧借用为协议输入；即使 FD 数据不超过 8 字节，也保留为 `Fd`。
///
/// `CanSocket` 返回的 `CanFrame` 可先通过 `CanAnyFrame::from(frame)` 包装；
/// `CanFdSocket` 返回的 `CanAnyFrame` 可直接传入。错误通知返回 `ErrorFrame`，
/// 不借用其载荷作协议解码，也不将其等同于 ODrive Heartbeat 中的设备错误位。
pub fn from_socketcan(frame: &CanAnyFrame) -> Result<FrameRef<'_>, FromSocketcanError> {
    let payload = match frame {
        CanAnyFrame::Normal(frame) => FramePayload::Data(frame.data()),
        CanAnyFrame::Remote(frame) => FramePayload::Remote {
            dlc: frame.dlc() as u8,
        },
        CanAnyFrame::Fd(frame) => FramePayload::Fd(frame.data()),
        CanAnyFrame::Error(_) => return Err(FromSocketcanError::ErrorFrame),
    };
    let id = match frame.id() {
        Id::Standard(id) => FrameId::Standard(id.as_raw()),
        Id::Extended(id) => FrameId::Extended(id.as_raw()),
    };
    Ok(FrameRef { id, payload })
}
