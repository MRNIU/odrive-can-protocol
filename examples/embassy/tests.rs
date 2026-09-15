// Copyright The odrive-can-protocol Contributors
//! 验证 Embassy 帧缓冲长度、RTR 与 FDF 的转换边界，不访问外设。

use adapter_check::adapter::*;
use embassy_stm32::can::frame::{FdFrame, Frame, Header};
use embedded_can::StandardId;
use odrive_can_protocol::{FramePayload, Message, NodeId, Query, encode};

#[test]
fn classic_frame_trims_padding_and_preserves_rtr() {
    let frame = Frame::new_standard(0x02e, &[0, 0, 0x20, 0xc0]).unwrap();
    assert_eq!(
        from_embassy(&frame).unwrap().payload,
        FramePayload::Data(&[0, 0, 0x20, 0xc0])
    );
    let encoded = encode(
        NodeId::new(1).unwrap(),
        Message::Request(Query::EncoderEstimates),
    )
    .unwrap();
    let frame = to_embassy(&encoded).unwrap();
    assert_eq!(
        from_embassy(&frame).unwrap().payload,
        FramePayload::Remote { dlc: 8 }
    );
    assert!(!frame.header().fdcan());
}

#[test]
fn short_fd_frames_keep_fdf_even_when_rtr_is_set() {
    let id = StandardId::new(0x021).unwrap().into();
    let frame = Frame::new(Header::new_fd(id, 8, true, false), &[0; 8]).unwrap();
    assert!(matches!(
        from_embassy(&frame).unwrap().payload,
        FramePayload::Fd(_)
    ));
    let frame = FdFrame::new(Header::new_fd(id, 8, false, false), &[0; 8]).unwrap();
    assert_eq!(
        from_embassy_fd(&frame).unwrap().payload,
        FramePayload::Fd(&[0; 8])
    );
}

#[test]
fn fd_container_can_carry_classic_frames_without_changing_wire_format() {
    let encoded = encode(
        NodeId::new(1).unwrap(),
        Message::Request(Query::EncoderEstimates),
    )
    .unwrap();
    let frame = to_embassy_fd(&encoded).unwrap();
    assert!(!frame.header().fdcan());
    assert_eq!(
        from_embassy_fd(&frame).unwrap().payload,
        FramePayload::Remote { dlc: 8 }
    );
    let frame = FdFrame::new_standard(0x02e, &[0, 0, 0x20, 0xc0]).unwrap();
    assert_eq!(
        from_embassy_fd(&frame).unwrap().payload,
        FramePayload::Data(&[0, 0, 0x20, 0xc0])
    );
}

#[test]
fn malformed_public_headers_return_error_without_panicking() {
    let id = StandardId::new(0x021).unwrap().into();
    let frame = Frame::new(Header::new(id, 9, false), &[]).unwrap();
    assert_eq!(
        from_embassy(&frame),
        Err(FromEmbassyError::InvalidPayloadLength)
    );
    let frame = FdFrame::new(Header::new_fd(id, 65, false, false), &[]).unwrap();
    assert_eq!(
        from_embassy_fd(&frame),
        Err(FromEmbassyError::InvalidPayloadLength)
    );
}
