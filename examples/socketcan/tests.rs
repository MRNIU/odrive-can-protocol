// Copyright The odrive-can-protocol Contributors
//! 用 SocketCAN 真实帧测试转换，包含错误帧和短 CAN FD，不打开 socket。

use adapter_check::adapter::*;
use embedded_can::{ExtendedId, Frame, StandardId};
use odrive_can_protocol::{Command, FrameId, FramePayload, Message, NodeId, Query, encode};
use socketcan::{CanAnyFrame, CanDataFrame, CanErrorFrame, CanFdFrame};

#[test]
fn socketcan_output_preserves_short_data_and_remote_dlc() {
    let node = NodeId::new(1).unwrap();
    let encoded = encode(
        node,
        Message::Command(Command::SetInputTorque { torque: -2.5 }),
    )
    .unwrap();
    let frame = to_socketcan(&encoded).unwrap();
    assert_eq!(frame.id(), StandardId::new(0x02e).unwrap().into());
    assert_eq!(frame.data(), &[0, 0, 0x20, 0xc0]);
    let encoded = encode(node, Message::Request(Query::EncoderEstimates)).unwrap();
    let frame = to_socketcan(&encoded).unwrap();
    assert!(frame.is_remote_frame());
    let any = CanAnyFrame::from(frame);
    assert_eq!(
        from_socketcan(&any).unwrap().payload,
        FramePayload::Remote { dlc: 8 }
    );
}

#[test]
fn socketcan_input_keeps_extended_id_and_short_fd_marker() {
    let frame =
        CanAnyFrame::Normal(CanDataFrame::new(ExtendedId::new(0x021).unwrap(), &[0; 8]).unwrap());
    assert_eq!(from_socketcan(&frame).unwrap().id, FrameId::Extended(0x021));
    let frame = CanAnyFrame::Fd(CanFdFrame::new(StandardId::new(0x021).unwrap(), &[0; 8]).unwrap());
    assert_eq!(
        from_socketcan(&frame).unwrap().payload,
        FramePayload::Fd(&[0; 8])
    );
}

#[test]
fn linux_error_frames_never_become_odrive_messages() {
    let frame = CanAnyFrame::Error(CanErrorFrame::new_error(0x021, &[0; 8]).unwrap());
    assert_eq!(from_socketcan(&frame), Err(FromSocketcanError::ErrorFrame));
}
