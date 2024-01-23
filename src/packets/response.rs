use std::{io::Cursor, str::FromStr};

use binrw::{NullString, BinRead, BinReaderExt, binread};

use crate::Error;

#[allow(clippy::no_effect_underscore_binding)]

#[derive(Debug, BinRead)]
pub struct PacketBase<T: BinRead<Args<'static> = ()>> {
    pub packet_type: u8,
    pub session_id: u32,
    pub payload: T,
}

impl<T: BinRead<Args<'static> = ()>> TryFrom<Vec<u8>> for PacketBase<T> {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut cur = Cursor::new(value);
        let response_packet = cur.read_be()?;
        Ok(response_packet)
    }
}


#[derive(Debug)]
#[binread]
pub struct Handshake {
    #[br(parse_with = nstring_int_parser)]
    pub challenge_token: u32,
}

#[derive(Debug, PartialEq, Eq)]
#[binread]
#[br(
    assert(&gametype == "SMP"),
)]
pub struct BasicStat {
    #[br(map = |x: NullString| x.to_string() )]
    pub motd: String,
    #[br(map = |x: NullString| x.to_string() )]
    pub gametype: String,
    #[br(map = |x: NullString| x.to_string() )]
    pub map: String,
    #[br(parse_with = nstring_int_parser)]
    pub numplayers: u16,
    #[br(parse_with = nstring_int_parser)]
    pub maxplayers: u16,
    #[br(little)]
    pub hostport: u16,
    #[br(map = |x: NullString| x.to_string() )]
    pub hostip: String,
}

#[derive(Debug, PartialEq, Eq)]
#[binread]
#[br(
    assert(&game_type == "SMP"),
    assert(&game_id == "MINECRAFT")
)]
pub struct FullStat {
    #[br(pad_before = 11, parse_with = kv_string_parser)]
    pub hostname: String,
    #[br(parse_with = kv_string_parser)]
    pub game_type: String,
    #[br(parse_with = kv_string_parser)]
    pub game_id: String,
    #[br(parse_with = kv_string_parser)]
    pub version: String,
    #[br(parse_with = kv_string_parser)]
    pub plugins: String,
    #[br(parse_with = kv_string_parser)]
    pub map: String,
    #[br(parse_with = kv_u16_parser)]
    pub numplayers: u16,
    #[br(parse_with = kv_u16_parser)]
    pub maxplayers: u16,
    #[br(parse_with = kv_u16_parser)]
    pub hostport: u16,
    #[br(parse_with = kv_string_parser)]
    pub hostip: String,

    #[br(
        count = numplayers as usize + 1,
        pad_before = 10,
        map(|x: Vec<NullString>| {
            x.into_iter()
                .filter_map(|s| { if s.len() > 0 { Some(s.to_string()) } else { None } })
                .collect()
        }),
    )]
    pub players: Vec<String>,
}

#[binrw::parser(reader, endian)]
fn kv_u16_parser() -> binrw::BinResult<u16> {
    _ = <NullString>::read_options(reader, endian, ())?;
    let nstring = <NullString>::read_options(reader, endian, ())?;
    let num = nstring
        .to_string()
        .parse::<u16>()
        .map_err(|err| binrw::Error::Custom { pos: 0, err: Box::new(err) })?;
    Ok(num)
}

#[binrw::parser(reader, endian)]
fn kv_string_parser() -> binrw::BinResult<String> {
    _ = <NullString>::read_options(reader, endian, ())?;
    let nstring = <NullString>::read_options(reader, endian, ())?;
    Ok(nstring.to_string())
}

#[binrw::parser(reader, endian)]
fn nstring_int_parser<T: FromStr>() -> binrw::BinResult<T> {
    let nstring = <NullString>::read_options(reader, endian, ())?;
    let num = nstring
        .to_string()
        .parse::<T>()
        .map_err(|_| binrw::Error::NoVariantMatch { pos: reader.stream_position().unwrap() })?;
    Ok(num)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{binread, BinReaderExt};

    use super::{kv_u16_parser, kv_string_parser, nstring_int_parser};

    #[test]
    fn kv_u16() {
        #[binread]
        struct Test {
            #[br(parse_with = kv_u16_parser)]
            value: u16,
        }

        let input = b"\x68\x6F\x73\x74\x6E\x61\x6D\x65\x3A\x00\x32\x35\x35\x36\x35\x00";
        
        let mut cur = Cursor::new(input);
        let test: Test = cur.read_be().unwrap();
        assert_eq!(test.value, 25565)
    }

    #[test]
    fn kv_string() {
        #[binread]
        struct Test {
            #[br(parse_with = kv_string_parser)]
            value: String,
        }

        let input = b"\x68\x6F\x73\x74\x6E\x61\x6D\x65\x3A\x00\x74\x65\x73\x74\x73\x65\x72\x76\x65\x72\x00";
        
        let mut cur = Cursor::new(input);
        let test: Test = cur.read_be().unwrap();
        assert_eq!(test.value, "testserver".to_string())
    }

    #[test]
    fn nstring_int() {
        #[binread]
        struct Test {
            #[br(parse_with = nstring_int_parser)]
            value: u32,
        }

        let input = b"\x30\x31\x32\x33\x34\x35\x36\x37\x00";
        
        let mut cur = Cursor::new(input);
        let test: Test = cur.read_be().unwrap();
        assert_eq!(test.value, 1234567)
    }

    #[test]
    fn nstring_int_error() {
        #[binread]
        struct Test {
            #[br(parse_with = nstring_int_parser)]
            _value: u32,
        }

        let input = b"\x30\x31\x32\x33\x34\x35\x36\x37\x41\x00";
        
        let mut cur = Cursor::new(input);
        let test: Result<Test, binrw::Error> = cur.read_be();
        assert!(test.is_err())
    }
}
