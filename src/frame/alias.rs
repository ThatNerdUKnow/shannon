#[cfg(not(feature = "sync_frame_channel"))]
use std::sync::mpsc::{Receiver, Sender};
#[cfg(feature = "sync_frame_channel")]
use std::sync::mpsc::{Receiver, SyncSender};

use super::Frame;

pub type FrameReceiver = Receiver<Frame>;

#[cfg(not(feature = "sync_frame_channel"))]
pub type FrameSender = Sender<Frame>;

#[cfg(feature = "sync_frame_channel")]
pub type FrameSender = SyncSender<Frame>;
