use ascii::AsciiChar;
use std::{
    io::{self, Write},
    mem::size_of,
};
use thiserror::Error;
pub mod parse;

#[derive(Clone,PartialEq,Debug)]
pub struct Frame<'a> {
    //n_bytes: u16,
    //target_user_id: u64,
    header: FrameHeader,
    body: FrameBody<'a>, //body: &'a [u8],
                         //crc32: u32,
}


impl<'a> Frame<'a> {
    /// Creates new [`Frame`] but does not verify that [`FrameHeader`] indicates correct number of bytes in the body
    /// or [`FrameBody`] has appropriate crc32 value
    pub fn new_unchecked(header: FrameHeader, body: FrameBody) -> Frame<'_> {
        Frame { header, body }
    }

    pub fn new(data: &[u8], user_id: u64) -> Result<Frame<'_>, FrameError> {
        let header = FrameHeader::new(data, user_id)?;
        let body = FrameBody::new(data);
        Ok(Frame { header, body })
    }
}

#[derive(Clone,PartialEq,Debug)]
pub struct FrameHeader {
    n_bytes: u16,
    target_user_id: u64,
}

#[derive(Clone,PartialEq,Debug)]
pub struct FrameBody<'a> {
    body: &'a [u8],
    crc32: u32,
}

impl FrameBody<'_> {
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
        FrameBody { body: data, crc32 }
    }

    /// Creates new [`FrameBody`] and automatically calculates crc32 value
    pub fn new(data: &[u8]) -> FrameBody {
        let crc = const_crc32::crc32(data);
        FrameBody::new_unchecked(data, crc)
    }
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
}

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("I/O Based Error")]
    IO(#[from] io::Error),
    #[error("CRC32 Mismatch. Calculated {0} but got {1}")]
    Crc32(u32, u32),
    #[error("Body size is too big. u16::MAX < {0}")]
    BodySize(usize),
}

impl<'a> TryFrom<Frame<'a>> for Vec<u8> {
    type Error = FrameError;

    fn try_from(frame: Frame<'a>) -> Result<Self, Self::Error> {
        let body_n_bytes: u16 = frame.header.n_bytes as u16;
        //let bytes_required = size_of::<u8>() // soh
        //    + size_of::<u16>() // size
        //    + size_of::<u64>()
        //    + size_of::<u8>() // stx
        //    + body_n_bytes as usize // body
        //    + size_of::<u32>() // crc
        //    + size_of::<u8>(); // etx

        let mut data: Vec<u8> = vec![0; 0];

        data.write(&[AsciiChar::SOH as u8])?;
        data.write(&body_n_bytes.to_be_bytes())?;
        data.write(&frame.header.target_user_id.to_be_bytes())?;
        data.write(&[AsciiChar::SOX as u8])?;
        data.write(frame.body.body)?;
        data.write(&frame.body.crc32.to_be_bytes())?;
        data.write(&[AsciiChar::ETX as u8])?;
        Ok(data)
    }
}

#[cfg(test)]
mod test{
    use super::Frame;

    #[test]
    fn new_frame(){
        Frame::new(&[0;64], 0).expect("Could not build frame");
    }

    #[test]
    fn frame_to_bytes(){
        let frame = Frame::new(&[0;64], 0).expect("Could not build frame");
        let _:Vec<u8> = frame.try_into().expect("Could not serialize frame");
    }

    #[test]
    fn reject_body_too_big(){
        let res = Frame::new(&[0;u16::MAX as usize + 1], 0);
        assert!(res.is_err())
    }
}