use std::io;

use super::error::FrameError;

#[derive(Clone, PartialEq, Debug)]
pub struct FrameHeader {
    n_bytes: u16,
    target_user_id: u64,
}

impl FrameHeader {
    pub fn new_unchecked(n_bytes: u16, user_id: u64) -> FrameHeader {
        FrameHeader {
            n_bytes,
            target_user_id: user_id,
        }
    }

    pub fn new(data: &[u8], user_id: u64) -> Result<FrameHeader, FrameError> {
        match data.len() > u16::MAX.into() {
            true => Err(FrameError::BodySize(data.len())),
            false => Ok(FrameHeader::new_unchecked(data.len() as u16, user_id)),
        }
    }

    pub fn write(&self,writer:&mut impl io::Write)->Result<(),io::Error>{
        writer.write_all(&self.n_bytes.to_be_bytes())?;
        writer.write_all(&self.target_user_id.to_be_bytes())?;
        Ok(())
    }

    pub fn n_bytes(&self)->u16{
        self.n_bytes
    }
}