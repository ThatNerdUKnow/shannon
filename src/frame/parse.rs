use ascii::AsciiChar;
use ascii::ToAsciiChar;
use log::error;
use nom::bytes;
use nom::error::ErrorKind;
use nom::number;
use nom::sequence;
use nom::sequence::tuple;
use nom::IResult;
use nom::combinator;

use crate::frame::FrameError;

use super::Frame;
use super::FrameBody;
use super::FrameHeader;

pub fn ascii_char(char: AsciiChar) -> impl Fn(&[u8]) -> IResult<&[u8], AsciiChar> {
    move |input: &[u8]| {
        let (remaining, parsed) = bytes::streaming::tag([char as u8])(input)?;
        //let out:AsciiChar = u8::to_ascii_char(parsed[0])?;
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

pub fn frame_body(header: FrameHeader) -> impl Fn(&[u8]) -> IResult<&[u8], FrameBody> {
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

pub fn frame(input: &[u8]) -> IResult<&[u8],Frame>{
    let body_parser = combinator::flat_map(frame_header, frame_body);
    let (remaining,(header,body)) = tuple((frame_header,body_parser))(input)?;
    Ok((remaining,Frame::new_unchecked(header, body)))
}

#[cfg(test)]
mod test{

}