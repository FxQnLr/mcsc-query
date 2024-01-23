use std::io::Cursor;

use binrw::{BinWrite, BinWriterExt};

use crate::Error;

use super::PacketType;

#[derive(Debug, BinWrite)]
#[bw(magic = b"\xFE\xFD")]
pub struct PacketBase<T: for<'a> BinWrite<Args<'a> = ()>> {
    pub packet_type: PacketType,
    pub session_id: u32,
    pub payload: T,
}

impl<T: for<'a> BinWrite<Args<'a> = ()>> TryFrom<PacketBase<T>> for Vec<u8> {
    type Error = Error;
    
    fn try_from(value: PacketBase<T>) -> Result<Self, Self::Error> {
        let mut cur = Cursor::new(vec![]);
        cur.write_be(&value)?;
        Ok(cur.into_inner())
    }
}

#[derive(Debug, BinWrite)]
pub struct Handshake;

#[derive(Debug, BinWrite)]
pub struct BasicStat {
    pub challenge_token: u32,
}

#[derive(Debug, BinWrite)]
pub struct FullStat {
    #[bw(pad_after = 4)]
    pub challenge_token: u32,
}

#[cfg(test)]
mod tests {
    use crate::packets::PacketType;

    use super::{FullStat, BasicStat, PacketBase};

    #[test]
    fn full_stat() {
        let s = PacketBase {
            packet_type: PacketType::Stat,
            session_id: 1,
            payload: FullStat {
                challenge_token: 0x0091295B,
            }
        };

        let mut output = std::io::Cursor::new(vec![]);
        binrw::BinWriterExt::write_be(&mut output, &s).unwrap();

        assert_eq!(
            output.into_inner(),
            vec![0xFE, 0xFD, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x91, 0x29, 0x5B, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn basic_stat() {
        let s = PacketBase {
            packet_type: PacketType::Stat,
            session_id: 1,
            payload: BasicStat {
                challenge_token: 0x0091295B,
            }
        };

        let mut output = std::io::Cursor::new(vec![]);
        binrw::BinWriterExt::write_be(&mut output, &s).unwrap();

        assert_eq!(
            output.into_inner(),
            vec![0xFE, 0xFD, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x91, 0x29, 0x5B]
        );
    }
}
