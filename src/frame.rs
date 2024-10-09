use anyhow::Result;
use embedded_can::nb::Can;
use socketcan::{CanFrame, CanSocket, Socket};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

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

    pub fn clear_captured_frames(&mut self) {
        match &mut self.captured_frames {
            CapturedFrames::List(l) => l.clear(),
            CapturedFrames::Set(s) => s.clear(),
        }
        self.total_frame_count = 0;
        self.frames_per_second = 0;
    }

    fn process_frame(&mut self, rx_frame: CanFrame) {
        match &mut self.captured_frames {
            CapturedFrames::List(l) => {
                l.push(rx_frame);
            }
            CapturedFrames::Set(_s) => {
                todo!("oklart")
            }
        }

        self.total_frame_count += 1;
    }

    fn update_frames_per_second(&mut self, tot_frames_last_second: usize) -> usize {
        if self.total_frame_count > tot_frames_last_second {
            self.frames_per_second = self.total_frame_count - tot_frames_last_second;
        }

        self.total_frame_count
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

    pub fn get_captured_frames_len(&mut self) -> usize {
        match &self.captured_frames.lock().unwrap().captured_frames {
            CapturedFrames::List(l) => l.len(),
            CapturedFrames::Set(s) => s.len(),
        }
    }

    // TODO: blir den här clone() för dyr?
    pub fn get_captured_frames(&mut self) -> CapturedFrames {
        let a = self.captured_frames.lock().unwrap();
        a.captured_frames.clone()
    }

    pub fn get_total_frame_count(&self) -> usize {
        self.captured_frames.lock().unwrap().total_frame_count
    }

    pub fn get_frames_per_second(&self) -> usize {
        self.captured_frames.lock().unwrap().frames_per_second
    }

    fn capture(mut rx_sock: CanSocket, a: Arc<Mutex<CapturedFrameState>>) {
        let mut running_second_timestamp = Instant::now();
        let mut tot_frames_as_of_last_second = 0;

        loop {
            if let Ok(rx_frame) = rx_sock.receive() {
                a.lock().unwrap().process_frame(rx_frame);
            }

            if running_second_timestamp.elapsed().as_secs() >= 1 {
                tot_frames_as_of_last_second = a
                    .lock()
                    .unwrap()
                    .update_frames_per_second(tot_frames_as_of_last_second);
                running_second_timestamp = Instant::now();
            }
        }
    }
}