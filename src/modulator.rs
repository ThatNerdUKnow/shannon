use std::{collections::VecDeque, io::{Read, Write}};


const AMPLITUDE_SCALE_FACTOR:u16 = u16::MAX / u8::MAX as u16;

pub struct PulseCodeModulator {
    buf: VecDeque<u8>,
}

impl Write for PulseCodeModulator{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // turn each incoming u8 into a big-endian u16 and pass into buf
        for el in buf{
            let el_16 = *el as u16 * AMPLITUDE_SCALE_FACTOR; 
            self.buf.write_all(&el_16.to_be_bytes())?;
        }
        Ok(buf.len()) // since our buf is in-memory none of the write_all calls should fail... I think
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(()) // pretty sure we don't need to do anything here
    }
}

impl Read for PulseCodeModulator{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.buf.read(buf)
    }
}