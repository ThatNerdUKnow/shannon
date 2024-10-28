use std::io;

use super::error::FrameError;

#[derive(Clone, PartialEq, Debug)]
pub struct FrameBody {
    body: Vec<u8>,
    crc32: u32,
}

impl FrameBody {
    /// Create new [`FrameBody`] with manual crc32 value. Provided crc32 is checked against the computed
    /// crc32 value for data
    pub fn new_checked(data: &[u8], crc32: u32) -> Result<FrameBody, FrameError> {
        let calc_crc = const_crc32::crc32(data);
        if calc_crc != crc32 {
            return Err(FrameError::Crc32(calc_crc, crc32));
        }
        Ok(FrameBody::new_unchecked(data, crc32))
    }

    /// Creates new [`FrameBody`] but does not verify provided crc32 value
    pub fn new_unchecked(data: &[u8], crc32: u32) -> FrameBody {
        FrameBody {
            body: Vec::from(data),
            crc32,
        }
    }

    /// Creates new [`FrameBody`] and automatically calculates crc32 value
    pub fn new(data: &[u8]) -> FrameBody {
        let crc = const_crc32::crc32(data);
        FrameBody::new_unchecked(data, crc)
    }

    pub fn write_raw(&self,writer:&mut impl io::Write)->Result<(),io::Error>{
        writer.write_all(&self.body)?;
        writer.write_all(&self.crc32.to_be_bytes())?;
        Ok(())
    }

    pub fn write_body(&self,writer:&mut impl io::Write)->Result<(),io::Error>{
        writer.write_all(&self.body)
    }
}