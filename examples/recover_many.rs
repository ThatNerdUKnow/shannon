use std::{collections::VecDeque, fmt::Write, fs, io::{self, BufReader, BufWriter, Read}, thread};

use log::info;
use rand::{thread_rng, Rng};
use shannon::frame::Frame;

pub fn main() {
    init();
    info!("Starting");
    let input = fs::File::open("./examples/berserk-ref.mkv").unwrap();
    let buf = BufReader::new(input);
    let  f = fs::File::create("./examples/out.mkv").unwrap();
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
    let stdout = BufWriter::with_capacity(u16::MAX as usize, io::stdout());
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Pipe(Box::new(stdout)))
        .try_init();
}
