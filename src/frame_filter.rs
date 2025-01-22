use crate::frame::TimestampedFrame;
use embedded_can::Frame;

type FrameFilterCallback = fn(frame: &TimestampedFrame, filter_ids: &[embedded_can::Id]) -> bool;

pub struct FrameIdFilter {
    pub ids: Vec<embedded_can::Id>,
    pub filter_callback: FrameFilterCallback,
}

pub fn filter_frame_on_ids(frame: &TimestampedFrame, filter_ids: &[embedded_can::Id]) -> bool {
    filter_ids
        .iter()
        .any(|filter_id| *filter_id == frame.frame.id())
}
