use embedded_can::nb::Can;
use embedded_can::Frame as EmbeddedFrame;
use std::collections::HashMap;

use std::hash::{Hash, Hasher};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use socketcan::{CanFrame, CanSocket, Frame};

#[derive(Clone)]
pub enum CapturedFrames {
    List(Vec<CanFrame>),
    Set(HashMap<u32, CanFrame>),
}

pub struct CapturedFrameState {
    captured_frames: CapturedFrames,
    total_frame_count: usize,
    frames_per_second: usize,
}

impl CapturedFrameState {
    pub fn new() -> Self {
        Self {
            captured_frames: CapturedFrames::List(vec![]),
            total_frame_count: 0,
            frames_per_second: 0,
        }
    }
}

pub struct FrameCaptor {
    //frame_info: Arc<Mutex<CapturedFrameInfo>>,
    captured_frames: CapturedFrameState,
    rx_socket: CanSocket,
    rxd_frames_channel: Sender<CapturedFrames>,
    // kommer nog behöva en Receiver för commandon också, t.ex. för att clear:a frames
}

impl FrameCaptor {
    //pub fn new(frame_info: Arc<Mutex<CapturedFrameInfo>>, rx_socket: CanSocket) -> Self {
    //    FrameCaptor {
    //       frame_info,
    //        rx_socket,
    //    }
    //}

    pub fn new(rx_socket: CanSocket, rxd_frames_channel: Sender<CapturedFrames>) -> Self {
        Self {
            captured_frames: CapturedFrameState::new(),
            rx_socket,
            rxd_frames_channel,
        }
    }

    pub fn clear_captured_frames(&mut self) {
        match &mut self.captured_frames.captured_frames {
            CapturedFrames::List(l) => l.clear(),
            CapturedFrames::Set(s) => s.clear(),
        }
        self.captured_frames.total_frame_count = 0;
        self.captured_frames.frames_per_second = 0;
    }

    //fn add_frame_to_captured_frame_set(
    //    &self,
    //    frame_info: &mut CapturedFrameInfo,
    //    mut captured_frame: CapturedFrame,
    //) {
    //    // Add to set
    //    if frame_info
    //        .captured_frame_set
    //        .contains_key(&captured_frame.id)
    //    {
    //        let old_count = frame_info
    //            .captured_frame_set
    //            .get(&captured_frame.id)
    //            .unwrap()
    //            .count;
    //        captured_frame.count = old_count + 1;
    //    }

    //    frame_info
    //        .captured_frame_set
    //        .insert(captured_frame.id, captured_frame);
    //}

    fn process_frame(&mut self, rx_frame: CanFrame) {
        match &mut self.captured_frames.captured_frames {
            CapturedFrames::List(l) => {
                l.push(rx_frame);
            }
            CapturedFrames::Set(_s) => {
                todo!("oklart")
            }
        }

        self.captured_frames.total_frame_count += 1;
    }

    /// Processes a received frame, adding it to the stored frame information
    //fn process_frame(&mut self, rx_frame: CanFrame) {
    //    let mut frame_info = self.frame_info.lock().unwrap();
    //    frame_info.total_frame_count += 1;

    //    let captured_frame = CapturedFrame::from_can_frame(rx_frame);

    //    // Kommer denna clone vara otroligt långsam?
    //    // Troligen inte så farlig, clone() på en frame tar inte så lång tid
    //    frame_info.captured_frame_vec.push(captured_frame.clone());

    //    // Add to set
    //    self.add_frame_to_captured_frame_set(&mut frame_info, captured_frame);
    //}

    fn update_frames_per_second(&mut self, tot_frames_last_second: usize) -> usize {
        self.captured_frames.frames_per_second =
            self.captured_frames.total_frame_count - tot_frames_last_second;

        self.captured_frames.total_frame_count
    }

    pub fn capture(mut self) {
        let mut running_second_timestamp = Instant::now();
        let mut tot_frames_as_of_last_second = 0;

        loop {
            if let Ok(rx_frame) = self.rx_socket.receive() {
                self.process_frame(rx_frame);
            }

            if running_second_timestamp.elapsed().as_secs() >= 1 {
                tot_frames_as_of_last_second =
                    self.update_frames_per_second(tot_frames_as_of_last_second);
                running_second_timestamp = Instant::now();
            }
        }
    }
}

// TODO: förhoppningsvis kan vi ta bort allt detta
// Represents information about a captured CAN frame
#[derive(Eq, PartialEq, Clone)]
pub struct CapturedFrame {
    pub id: u32,
    pub dlc: usize,
    pub count: usize,
    pub is_extended: bool,
    pub data: [u8; 8],
    pub as_ascii: [u8; 8],
}

// Only hash on ID, as we discriminate CAN frames based on ID
impl Hash for CapturedFrame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        state.finish();
    }
}

impl CapturedFrame {
    fn from_can_frame(frame: CanFrame) -> Self {
        let mut data = [0; 8];

        data[..frame.data().len()].copy_from_slice(frame.data());

        Self {
            id: frame.raw_id(),
            dlc: frame.dlc(),
            count: 1,
            is_extended: frame.is_extended(),
            data,
            as_ascii: [0; 8],
        }
    }

    pub fn get_data_string(&self) -> String {
        format!("{:#04x?}", self.data).replace(['[', ']', ',', '\n'], "")
    }
}
