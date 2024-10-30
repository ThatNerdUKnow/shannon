use std::{
    collections::VecDeque,
    io::{self, Read},
    sync::mpsc::Receiver,
};

use log::{debug, error, info, warn};

use super::Frame;

pub struct FrameReader {
    rx: Receiver<Option<Frame>>,
    buf: VecDeque<u8>,
    user_id: u64,
    frame_count: usize,
}

impl FrameReader {
    pub fn new(rx: Receiver<Option<Frame>>, user_id: u64) -> FrameReader {
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
        match self.rx.recv() {
            Ok(Some(frame)) => {
                let frame_uid = frame.header.user_id();
                if frame_uid == self.user_id {
                    self.frame_count += 1;
                    debug!(
                        "Recieved frame #{} for user id {}",
                        self.frame_count, self.user_id
                    );
                    frame.write_body(&mut self.buf)?;
                    self.buf.read(buf)
                } else {
                    warn!(
                        "Recieved frame for unknown user id {}. expected {}",
                        frame_uid, self.user_id
                    );
                    Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        format!("skipped frame with user id {frame_uid}"),
                    ))
                }
            }
            Ok(None) => {
                info!("Rx stream ended for user id{}", self.user_id);
                Ok(0)
            }
            Err(e) => {
                warn!("{e}");
                Ok(0)
            }
        }
    }
}
