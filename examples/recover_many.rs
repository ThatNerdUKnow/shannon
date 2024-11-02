use std::{collections::VecDeque, fmt::Write, fs, io::{self, BufReader, BufWriter, Read}, thread::{self, Thread}};

use log::info;
use rand::{thread_rng, Rng};
use shannon::frame::Frame;

pub fn main() {
    init();
    info!("Starting");
    let in_str = "./examples/moby.txt";
    
    let t_raw = thread::spawn(||process_raw(in_str, "./examples/out_raw.txt"));
    let t_body = thread::spawn(||process_body(in_str, "./examples/out_body.txt"));
    t_raw.join().unwrap();
    t_body.join().unwrap();
}

fn process_body(in_str:&str,out_str:&str){
    let input = fs::File::open(in_str).unwrap();
    let buf = BufReader::new(input);
    let  f = fs::File::create(out_str).unwrap();
    let mut buf2 = BufWriter::new(f);
    let user_id: u64 = thread_rng().gen();
    info!("Writing frames to channel");
    let rx = Frame::write(buf, user_id);
    info!("Reading body from stream");
    let mut rdr = Frame::read_body_from_stream(rx, user_id);
    info!("reading final buf");
    std::io::copy(&mut rdr, &mut buf2).unwrap();
}

fn process_raw(in_str:&str,out_str:&str){
    let input = fs::File::open(in_str).unwrap();
    let buf = BufReader::new(input);
    let  f = fs::File::create(out_str).unwrap();
    let mut buf2 = BufWriter::new(f);
    let user_id: u64 = thread_rng().gen();
    info!("Writing frames to channel");
    let rx = Frame::write(buf, user_id);
    info!("Reading body from stream");
    let mut rdr = Frame::read_raw_from_stream(rx, user_id);
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
