use super::{error::FrameError, Frame};

impl TryFrom<Frame> for Vec<u8> {
    type Error = FrameError;

    fn try_from(frame: Frame) -> Result<Self, Self::Error> {
        let mut data = vec![0; 0];
        frame.write_frame(&mut data)?;
        Ok(data)
    }
}