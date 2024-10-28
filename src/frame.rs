use std::io;

use ascii::AsciiChar;
use body::FrameBody;
use error::FrameError;
use header::FrameHeader;
pub mod body;
pub mod error;
pub mod header;
mod impls;
pub mod parse;

#[derive(Clone, PartialEq, Debug)]
pub struct Frame {
    //n_bytes: u16,
    //target_user_id: u64,
    header: FrameHeader,
    body: FrameBody, //body: &'a [u8],
                     //crc32: u32,
}

impl Frame {
    /// Creates new [`Frame`] but does not verify that [`FrameHeader`] indicates correct number of bytes in the body
    /// or [`FrameBody`] has appropriate crc32 value
    pub fn new_unchecked(header: FrameHeader, body: FrameBody) -> Frame {
        Frame { header, body }
    }

    pub fn new(data: &[u8], user_id: u64) -> Result<Frame, FrameError> {
        let header = FrameHeader::new(data, user_id)?;
        let body = FrameBody::new(data);
        Ok(Frame { header, body })
    }

    #[cfg(test)]
    fn bytes_required(&self) -> usize {
        let body_n_bytes: u16 = self.header.n_bytes() as u16;
        let bytes_required = size_of::<u8>() // soh
            + size_of::<u16>() // size
            + size_of::<u64>()
            + size_of::<u8>() // stx
            + body_n_bytes as usize // body
            + size_of::<u32>() // crc
            + size_of::<u8>(); // etx
        return bytes_required;
    }

    fn write_frame(&self, data: &mut impl io::Write) -> Result<(), FrameError> {
        data.write_all(&[AsciiChar::SOH as u8])?;
        self.header.write_raw(data)?;
        data.write_all(&[AsciiChar::SOX as u8])?;
        self.body.write_raw(data)?;
        data.write_all(&[AsciiChar::ETX as u8])?;
        Ok(())
    }

    fn write_body(&self, data: &mut impl io::Write) -> Result<(), FrameError> {
        self.body.write_body(data)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Frame;

    #[test]
    fn new_frame() {
        Frame::new(&[0; 64], 0).expect("Could not build frame");
    }

    #[test]
    fn frame_to_bytes() {
        let frame = Frame::new(&[0; 64], 0).expect("Could not build frame");
        let v: Vec<u8> = frame.clone().try_into().expect("Could not serialize frame");
        println!("Written bytes length does not match expected");
        assert_eq!(v.len(), frame.bytes_required());
    }

    #[test]
    fn reject_body_too_big() {
        let res = Frame::new(&[0; u16::MAX as usize + 1], 0);
        assert!(res.is_err())
    }
}
