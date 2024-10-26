use ascii::AsciiChar;
use std::{
    io::{self, Write},
    mem::size_of,
};
use thiserror::Error;
pub mod parse;
pub struct Frame<'a> {
    //n_bytes: u16,
    //target_user_id: u64,
    header: FrameHeader,
    body:FrameBody<'a>
    //body: &'a [u8],
    //crc32: u32,
}

pub struct FrameHeader {
    n_bytes: u16,
    target_user_id: u64,
}

pub struct FrameBody<'a>{
    body: &'a[u8],
    crc32:u32
}

impl FrameBody<'_>{
    /// Create new [`FrameBody`] with manual crc32 value. Provided crc32 is checked against the computed
    /// crc32 value for data
    pub fn new_checked(data:&[u8],crc32:u32)->Result<FrameBody,FrameError>{
        let calc_crc = const_crc32::crc32(data);
        if calc_crc != crc32{
            return Err(FrameError::Crc32(calc_crc, crc32))
        }
        Ok(FrameBody{
            body:data,
            crc32
        })
    }
}

impl FrameHeader {
    pub fn new(n_bytes: u16, user_id: u64) -> FrameHeader {
        FrameHeader {
            n_bytes,
            target_user_id: user_id,
        }
    }
}

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("I/O Based Error")]
    IO(#[from] io::Error),
    #[error("CRC32 Mismatch. Calculated {0} but got {1}")]
    Crc32(u32,u32)
}


impl<'a> TryFrom<Frame<'a>> for Vec<u8> {
    type Error = FrameError;

    fn try_from(frame: Frame<'a>) -> Result<Self, Self::Error> {
        let body_n_bytes: u16 = frame.header.n_bytes as u16;
        let bytes_required = size_of::<u8>() // soh
            + size_of::<u16>() // size
            + size_of::<u64>()
            + size_of::<u8>() // stx
            + body_n_bytes as usize // body
            + size_of::<u32>() // crc
            + size_of::<u8>(); // etx

        let mut data: Vec<u8> = vec![0; bytes_required];

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
