use std::{io::{self, Write}, mem::size_of};
use ascii::AsciiChar;
use byteorder::{ByteOrder, NetworkEndian};
use thiserror::Error;

pub struct Frame<'a> {
    n_bytes: u16,
    target_user_id: u64,
    body: &'a [u8],
    crc32: u32,
}

#[derive(Error,Debug)]
pub enum FrameError{
    #[error("I/O Based Error")]
    IO(#[from] io::Error)
}

impl<'a> TryFrom<Frame<'a>> for Vec<u8> {
    type Error = FrameError;
    
    
    fn try_from(frame: Frame<'a>) -> Result<Self, Self::Error> {
        
        let body_n_bytes:u16 = frame.n_bytes as u16;
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
        data.write(&frame.target_user_id.to_be_bytes())?;
        data.write(&[AsciiChar::SOX as u8])?;
        data.write(frame.body)?;
        data.write(&frame.crc32.to_be_bytes())?;
        data.write(&[AsciiChar::ETX as u8])?;
        Ok(data)
    }
}
