use std::io::Read;

use ascii::AsciiChar;
use ascii::ToAsciiChar;
use log::error;
use nom::bytes;
use nom::error::ErrorKind;
use nom::number;
use nom::sequence;
use nom::sequence::terminated;
use nom::IResult;

use crate::frame::FrameError;

use super::Frame;
use super::FrameBody;
use super::FrameHeader;

pub fn ascii_char(char: AsciiChar) -> impl Fn(&[u8]) -> IResult<&[u8], AsciiChar> {
    move |input: &[u8]| {
        let (remaining, parsed) = bytes::streaming::tag([char as u8])(input)?;
        match u8::to_ascii_char(parsed[0]) {
            Ok(out) => Ok((remaining, out)),
            Err(err) => {
                error!("{err}");
                Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Char,
                )))
            }
        }
    }
}

pub fn frame_header(input: &[u8]) -> IResult<&[u8], FrameHeader> {
    let sequence = sequence::pair(number::streaming::be_u16, number::streaming::be_u64);
    let (remaining, (n_bytes, user_id)) = sequence::delimited(
        ascii_char(AsciiChar::SOH),
        sequence,
        ascii_char(AsciiChar::SOX),
    )(input)?;
    let header = FrameHeader::new_unchecked(n_bytes, user_id);
    Ok((remaining, header))
}

pub fn frame_body<'a>(
    header: FrameHeader,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], FrameBody> {
    move |input: &[u8]| {
        let (remaining, (data, crc)) = sequence::tuple((
            bytes::streaming::take::<u16, &[u8], nom::error::Error<&[u8]>>(header.n_bytes),
            number::streaming::be_u32,
        ))(input)?;

        match FrameBody::new_checked(data, crc) {
            Ok(body) => Ok((remaining, body)),
            Err(err) => match err {
                FrameError::Crc32(_, _) => {
                    error!("{err}");
                    Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        ErrorKind::Verify,
                    )))
                }
                _ => unreachable!(),
            },
        }
    }
}

pub fn frame(input: &[u8]) -> IResult<&[u8], Frame> {
    let (remaining, header) = frame_header(input)?;
    let (remaining, body) =
        terminated(frame_body(header.clone()), ascii_char(AsciiChar::ETX))(remaining)?;
    Ok((remaining, Frame::new_unchecked(header, body)))
}

#[cfg(test)]
mod test {
    use rand::{thread_rng, Rng};

    use crate::frame::Frame;
    const BUF_SIZE: usize = 4096;
    fn rand_buf() -> [u8; BUF_SIZE] {
        let mut rng = rand::thread_rng();
        let mut data: [u8; BUF_SIZE] = [0; BUF_SIZE];
        rng.fill(&mut data);
        data
    }

    #[test]
    fn frame_recovery() {
        let buf = rand_buf();
        let user_id: u64 = thread_rng().gen();
        let frame = Frame::new(&buf, user_id).expect("Could not create frame");
        let raw: Vec<u8> = frame.clone().try_into().expect("Could not serialize frame");
        let (remaining, recovered) = super::frame(&raw).expect("Could not parse frame");
        assert_eq!(remaining.len(), 0);
        assert_eq!(frame, recovered);
    }
}
