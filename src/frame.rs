use embedded_can::nb::Can;
use embedded_can::Frame as EmbeddedFrame;
use std::collections::HashMap;

use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use socketcan::{CanFrame, CanSocket, Frame};

pub struct CapturedFrameInfo {
    pub captured_frame_vec: Vec<CapturedFrame>,
    pub captured_frame_set: HashMap<u32, CapturedFrame>,
    pub total_frame_count: usize,
    pub frames_per_second: usize,
}

impl CapturedFrameInfo {
    pub fn new() -> Self {
        Self {
            captured_frame_vec: vec![],
            captured_frame_set: HashMap::new(),
            total_frame_count: 0,
            frames_per_second: 0,
        }
    }

    pub fn clear_captured_frames(&mut self) {
        self.captured_frame_vec = vec![];
        self.captured_frame_set = HashMap::new();
        self.total_frame_count = 0;
        self.frames_per_second = 0;
    }
}

impl Default for CapturedFrameInfo {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FrameCaptor {
    frame_info: Arc<Mutex<CapturedFrameInfo>>,
    rx_socket: CanSocket,
}

impl FrameCaptor {
    pub fn new(frame_info: Arc<Mutex<CapturedFrameInfo>>, rx_socket: CanSocket) -> Self {
        FrameCaptor {
            frame_info,
            rx_socket,
        }
    }

    fn add_frame_to_captured_frame_set(
        &self,
        frame_info: &mut CapturedFrameInfo,
        mut captured_frame: CapturedFrame,
    ) {
        // Add to set
        if frame_info
            .captured_frame_set
            .contains_key(&captured_frame.id)
        {
            let old_count = frame_info
                .captured_frame_set
                .get(&captured_frame.id)
                .unwrap()
                .count;
            captured_frame.count = old_count + 1;
        }

        frame_info
            .captured_frame_set
            .insert(captured_frame.id, captured_frame);
    }

    /// Processes a received frame, adding it to the stored frame information
    fn process_frame(&mut self, rx_frame: CanFrame) {
        let mut frame_info = self.frame_info.lock().unwrap();
        frame_info.total_frame_count += 1;

        let captured_frame = CapturedFrame::from_can_frame(rx_frame);

        // Kommer denna clone vara otroligt långsam?
        frame_info.captured_frame_vec.push(captured_frame.clone());

        // Add to set
        self.add_frame_to_captured_frame_set(&mut frame_info, captured_frame);
    }

    fn update_frames_per_second(&mut self, tot_frames_last_second: &mut usize) {
        let mut frame_info = self.frame_info.lock().unwrap();
        frame_info.frames_per_second = frame_info.total_frame_count - *tot_frames_last_second;
        *tot_frames_last_second = frame_info.total_frame_count;
    }

    pub fn capture(mut self) {
        let mut running_second_timestamp = Instant::now();
        let mut tot_frames_last_second = 0;
        loop {
            if let Ok(rx_frame) = self.rx_socket.receive() {
                self.process_frame(rx_frame);
            }

            if running_second_timestamp.elapsed().as_secs() >= 1 {
                self.update_frames_per_second(&mut tot_frames_last_second);
                running_second_timestamp = Instant::now();
            }
        }
    }
}

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
