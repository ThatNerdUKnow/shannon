use std::{
    cmp::min, collections::VecDeque, io::{self, Read, Write}, sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    }, thread
};

use ascii::AsciiChar;
use body::FrameBody;
use error::FrameError;
use framereader::FrameReader;
use header::FrameHeader;
use log::{debug, error, info, trace, warn, Level};
pub mod body;
pub mod error;
pub mod framereader;
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
        use std::mem::size_of;

        let body_n_bytes: u16 = self.header.n_bytes();
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

    fn write_body(&self, data: &mut impl io::Write) -> Result<(), io::Error> {
        self.body.write_body(data)?;
        Ok(())
    }

    fn flush_frame(
        n_bytes: usize,
        user_id: u64,
        buf: &mut Vec<u8>,
        tx: &Sender<Frame>,
    ) -> Result<(), FrameError> {
        let drain:Vec<u8> = buf.drain(0..n_bytes).collect();
        let frame = Frame::new(&drain, user_id)?;

        Ok(tx.send(frame)?)
    }

    /// Write the contents of a reader into (potentially many) frames
    pub fn write<T: Read + Send + Sync + 'static>(mut reader: T, user_id: u64) -> Receiver<Frame> {
        let (tx, rx) = mpsc::channel::<Frame>();
        let thread_tx = tx.clone();
        info!("Spawning thread");
        thread::spawn(move || {
            let mut flex_buf: Vec<u8> = vec![0; 0];
            let mut buf = [0; u16::MAX as usize];
            info!("Frame write thread for user id {user_id}");
            let mut n_frames = 0;
            
            while let Ok(count) = reader.read(&mut buf){
                debug!("Read {count} bytes from reader");
                if count == 0{
                    break;
                }

                flex_buf.write_all(&buf[0..count]).expect("Couldn't write to flex_buf");

                
            }

            warn!("{} bytes remining in flex_buf",flex_buf.len());
            // flush the rest of the frame buffer
            while !flex_buf.is_empty(){
                let buf_len = min(u16::MAX as usize, flex_buf.len());
                Frame::flush_frame(buf_len, user_id, &mut flex_buf, &thread_tx).inspect_err(|e|error!("{e}")).unwrap();
            }
        });
        debug!("Returned rx");
        rx
    }

    pub fn read_body_from_stream(rx: Receiver<Frame>, user_id: u64) -> impl io::Read {
        FrameReader::new(rx, user_id)
    }
}

#[cfg(test)]
mod test {
    use std::{collections::VecDeque, io::Read};

    use log::{info, log};
    use rand::{thread_rng, Rng};

    use super::Frame;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

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

    #[test]
    fn channel_one_frame() {
        init();
        let buf: VecDeque<u8> = VecDeque::from(vec![0x0f; (u16::MAX as usize)]);
        let user_id: u64 = 0x89ABCDEF;
        let rx = Frame::write(buf.clone(), user_id);
        let recovered = rx.recv().expect("Expected at least one frame");

        assert_eq!(&buf, &recovered.body.body());
    }

    #[test]
    fn recover_many() {
        init();
        let buf= include_bytes!(".././examples/moby.txt");
        let user_id: u64 = thread_rng().gen();
        let rx = Frame::write(&buf[..], user_id);
        let mut rdr = Frame::read_body_from_stream(rx, user_id);
        let mut buf2: Vec<u8> = vec![0; 0];
        rdr.read_to_end(&mut buf2).expect("Buf read failed");
        assert_eq!(buf.len(),buf2.len());
        assert_eq!(&buf[..], &buf2);
    }
}
