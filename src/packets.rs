use binrw::{BinRead, BinWrite};

pub mod response;
pub mod request;

#[derive(Debug, BinRead, BinWrite)]
#[brw(repr = u8)]
pub enum PacketType {
    Stat = 0,
    Handshake = 9
}
