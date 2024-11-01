use std::{io, sync::mpsc::SendError};

use thiserror::Error;

use super::Frame;

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("I/O Based Error")]
    IO(#[from] io::Error),
    #[error("CRC32 Mismatch. Calculated {0} but got {1}")]
    Crc32(u32, u32),
    #[error("Body size is too big. u16::MAX < {0}")]
    BodySize(usize),
    #[error("{0}")]
    Send(#[from] SendError<Frame>)
}