use anyhow::Result;
use embedded_can::nb::Can;
use embedded_can::Frame;
use socketcan::{CanFrame, CanSocket, Socket};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct TimestampedFrame {
    pub frame: CanFrame,
    timestamp: SystemTime,
    pub frame_number: u64,
}

impl TimestampedFrame {
    pub fn new(frame: CanFrame, frame_number: u64) -> Self {
        Self {
            frame,
            timestamp: SystemTime::now(),
            frame_number,
        }
    }

    pub fn get_timestamp(&self) -> u128 {
        self.timestamp
            .duration_since(UNIX_EPOCH)
            .expect("Error unwrapping duration since epoch!")
            .as_millis()
    }

    pub fn get_numeric_id(&self) -> u32 {
        match CanFrame::id(&self.frame) {
            socketcan::Id::Standard(standard_id) => standard_id.as_raw() as u32,
            socketcan::Id::Extended(extended_id) => extended_id.as_raw(),
        }
    }
}

pub struct CountedFrame {
    pub frame: CanFrame,
    pub capture_count: usize,
}

impl CountedFrame {
    pub fn new(frame: CanFrame) -> Self {
        Self {
            frame,
            capture_count: 0,
        }
    }

    pub fn with_capture_count(frame: CanFrame, capture_count: usize) -> Self {
        Self {
            frame,
            capture_count,
        }
    }
}

pub struct CapturedFrameState {
    pub captured_frames_list: Vec<TimestampedFrame>,
    pub captured_frames_set: HashMap<u32, CountedFrame>,
    total_frame_count: usize,
    frames_per_second: usize,
    frames_per_second_history: Vec<(f64, SystemTime)>,
}

impl Default for CapturedFrameState {
    fn default() -> Self {
        Self::new()
    }
}

impl CapturedFrameState {
    pub fn new() -> Self {
        Self {
            captured_frames_list: vec![],
            captured_frames_set: HashMap::new(),
            total_frame_count: 0,
            frames_per_second: 0,
            frames_per_second_history: vec![],
        }
    }

    pub fn clear_captured_frames(&mut self) {
        self.captured_frames_list.clear();
        self.captured_frames_set.clear();
        self.total_frame_count = 0;
        self.frames_per_second = 0;
    }

    fn process_frame(&mut self, rx_frame: CanFrame, frame_number: u64) {
        self.captured_frames_list
            .push(TimestampedFrame::new(rx_frame, frame_number));

        let old_capture_count = self
            .captured_frames_set
            .entry(socketcan::Frame::raw_id(&rx_frame))
            .or_insert(
                CountedFrame::new(rx_frame), //(rx_frame, 0)
            )
            .capture_count;

        self.captured_frames_set.insert(
            socketcan::Frame::raw_id(&rx_frame),
            CountedFrame::with_capture_count(rx_frame, old_capture_count + 1),
        );

        self.total_frame_count += 1;
    }

    /* Updates the number of frames per second, as seen by the Frame Captor

       # Arguments

       * `tot_frames_as_of_last_second` - The total number of frames seen on
          the bus, as of the last second.
    */
    fn update_frames_per_second(&mut self, tot_frames_as_of_last_second: usize) {
        if self.total_frame_count > tot_frames_as_of_last_second {
            self.frames_per_second = self.total_frame_count - tot_frames_as_of_last_second;
            let timestamp = SystemTime::now();
            self.frames_per_second_history
                .push((self.frames_per_second as f64, timestamp));
        }
    }
}

pub struct FrameCaptor {
    captured_frames: Arc<Mutex<CapturedFrameState>>,
    _capture_thread_handle: std::thread::JoinHandle<()>,
}

impl FrameCaptor {
    pub fn new(can_interface: String) -> Result<Self> {
        let rx_sock = CanSocket::open(can_interface.as_str())?;
        let cap_frame_state = Arc::new(Mutex::new(CapturedFrameState::new()));
        let thread_cap_frame_state = Arc::clone(&cap_frame_state);

        let _capture_thread_handle =
            std::thread::spawn(move || FrameCaptor::capture(rx_sock, thread_cap_frame_state));

        Ok(Self {
            captured_frames: cap_frame_state,
            _capture_thread_handle,
        })
    }

    pub fn clear_captured_frames(&mut self) {
        let mut b = self.captured_frames.lock().unwrap();
        b.clear_captured_frames();
    }

    pub fn get_captured_frames_list_len(&mut self) -> usize {
        self.captured_frames
            .lock()
            .unwrap()
            .captured_frames_list
            .len()
    }

    pub fn get_captured_frames_set_len(&mut self) -> usize {
        self.captured_frames
            .lock()
            .unwrap()
            .captured_frames_set
            .len()
    }

    pub fn get_captured_frames(&mut self) -> Arc<Mutex<CapturedFrameState>> {
        Arc::clone(&self.captured_frames)
    }

    pub fn get_unique_frame_count(&mut self) -> usize {
        let frames = self.captured_frames.lock().unwrap();
        frames.captured_frames_set.len()
    }

    pub fn get_total_frame_count(&self) -> usize {
        self.captured_frames.lock().unwrap().total_frame_count
    }

    pub fn get_frames_per_second(&self) -> usize {
        self.captured_frames.lock().unwrap().frames_per_second
    }

    pub fn get_frames_per_second_history(&self) -> Vec<(f64, f64)> {
        let now = SystemTime::now();

        let frame_rate_history = self
            .captured_frames
            .lock()
            .unwrap()
            .frames_per_second_history
            .clone();

        frame_rate_history
            .iter()
            .map(|(fps, timestamp)| {
                (
                    now.duration_since(*timestamp).unwrap().as_secs() as f64,
                    *fps,
                )
            })
            .collect()
    }

    fn capture(mut rx_sock: CanSocket, frame_state: Arc<Mutex<CapturedFrameState>>) {
        let mut running_second_timestamp = Instant::now();
        let mut tot_frames_as_of_last_second = 0;
        let mut frame_number: u64 = 0;

        loop {
            if let Ok(rx_frame) = rx_sock.receive() {
                frame_state
                    .lock()
                    .unwrap()
                    .process_frame(rx_frame, frame_number);
                frame_number += 1;
            }

            if running_second_timestamp.elapsed().as_secs() >= 1 {
                let mut f = frame_state.lock().unwrap();

                f.update_frames_per_second(tot_frames_as_of_last_second);
                tot_frames_as_of_last_second = f.total_frame_count;

                running_second_timestamp = Instant::now();
            }
        }
    }
}
