use std::{collections::VecDeque, fmt::Write, fs, io::{self, BufReader, BufWriter, Read}, thread};

use log::info;
use rand::{thread_rng, Rng};
use shannon::frame::Frame;

pub fn main() {
    env_logger::init();
    info!("Starting");
    let input = fs::File::open("./examples/moby.txt").unwrap();
    let buf = BufReader::new(input);
    let  f = fs::File::create("./examples/out.txt").unwrap();
    let mut buf2 = BufWriter::new(f);
    let user_id: u64 = thread_rng().gen();
    info!("Writing frames to channel");
    let rx = Frame::write(buf, user_id);
    info!("Reading body from stream");
    let mut rdr = Frame::read_body_from_stream(rx, user_id);
    info!("reading final buf");
    std::io::copy(&mut rdr, &mut buf2).unwrap();

}

fn init() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();
}
