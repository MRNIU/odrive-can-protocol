// Copyright The odrive-can-protocol Contributors
//! 用 bxCAN 真实帧验证通用 Classic CAN 适配，不访问外设。

use adapter_check::adapter::{from_embedded_can, to_embedded_can};
use embedded_can::{ExtendedId, Frame, StandardId};
use odrive_can_protocol::{Command, FrameId, FramePayload, Message, NodeId, Query, encode};

#[test]
fn native_bxcan_preserves_wire_bytes_and_rtr_dlc() {
    let node = NodeId::new(1).unwrap();
    let encoded = encode(
        node,
        Message::Command(Command::SetInputTorque { torque: -2.5 }),
    )
    .unwrap();
    let native: bxcan::Frame = to_embedded_can(&encoded).unwrap();
    assert_eq!(Frame::id(&native), StandardId::new(0x02e).unwrap().into());
    assert_eq!(Frame::data(&native), &[0x00, 0x00, 0x20, 0xc0]);
    let query = encode(node, Message::Request(Query::EncoderEstimates)).unwrap();
    let native: bxcan::Frame = to_embedded_can(&query).unwrap();
    assert!(native.is_remote_frame());
    assert_eq!(native.dlc(), 8);
    assert_eq!(
        from_embedded_can(&native).unwrap().payload,
        FramePayload::Remote { dlc: 8 }
    );
}

#[test]
fn received_bxcan_borrows_only_valid_bytes_and_preserves_extended_id() {
    let native = <bxcan::Frame as Frame>::new(
        ExtendedId::new(0x021).unwrap(),
        &[0x80, 0, 0, 0, 8, 0, 0, 1],
    )
    .unwrap();
    let view = from_embedded_can(&native).unwrap();
    assert_eq!(view.id, FrameId::Extended(0x021));
    assert_eq!(
        view.payload,
        FramePayload::Data(&[0x80, 0, 0, 0, 8, 0, 0, 1])
    );
    let empty = <bxcan::Frame as Frame>::new(StandardId::new(0x022).unwrap(), &[]).unwrap();
    assert_eq!(
        from_embedded_can(&empty).unwrap().payload,
        FramePayload::Data(&[])
    );
}
