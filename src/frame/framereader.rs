use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    sync::mpsc::Receiver,
};

use log::{debug, error, info, trace, warn, Level};

use super::Frame;

pub struct FrameReader {
    rx: Receiver<Frame>,
    buf: VecDeque<u8>,
    user_id: u64,
    frame_count: usize,
}

impl FrameReader {
    pub fn new(rx: Receiver<Frame>, user_id: u64) -> FrameReader {
        FrameReader {
            rx,
            user_id,
            buf: VecDeque::new(),
            frame_count: 0,
        }
    }
}

impl Read for FrameReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        debug!("Requested {} bytes from buf", buf.len());
        if buf.len() > self.buf.len() {
            if let Ok(frame) = self.rx.recv().inspect_err(|e| error!("{e}")) {
                self.frame_count += 1;
                info!(
                    "Got frame #{} for user id {}",
                    self.frame_count, self.user_id
                );
                if frame.header.user_id() == self.user_id {
                    self.buf.write_all(frame.body.body())?;
                }
            }
        } else {
            debug!("Already enough bytes in buf. skipping channel read");
        }
        let count = self.buf.read(buf)?;
        debug!("Read {} bytes from buf", count);
        debug!("{} bytes remaining in buf", self.buf.len());
        Ok(count)
    }
}
