use std::{collections::VecDeque, fmt::Write, fs, io::{self, BufReader, Read}, thread};

use log::info;
use rand::{thread_rng, Rng};
use shannon::frame::Frame;

pub fn main() {
    init();
    info!("Starting");
    //let buf: VecDeque<u8> = VecDeque::from(vec![0x0f; (u32::MAX as usize)]);
    let f = fs::File::open("./examples/moby.txt").unwrap();
    let buf = BufReader::new(f);
    let mut buf2 = fs::File::create("./examples/out.txt").unwrap();
    let user_id: u64 = thread_rng().gen();
    info!("Writing frames to channel");
    let rx = Frame::write(buf, user_id);
    info!("Reading body from stream");
    let mut rdr = Frame::read_body_from_stream(rx, user_id);
    //let mut buf2: Vec<u8> = vec![0; 0];
    info!("reading final buf");
    std::io::copy(&mut rdr, &mut buf2).unwrap();
    //rdr.read_to_end(&mut buf2).expect("Buf read failed");

}

fn init() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();
}
