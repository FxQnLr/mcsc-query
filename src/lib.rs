use std::net::{UdpSocket, ToSocketAddrs};

use packets::{request, response, PacketType};

mod packets;

pub use packets::response::{BasicStat, FullStat};

/// Gets a struct with the basic set of stats collectible by the query protocol
///
/// # Errors
///
/// - **Io** ([`std::io::Error`]) from the [`std::net::UdpSocket`] and writers
/// - **Conversion** ([`std::num::ParseIntError`]) from the query packet conversion
/// - **Conversion** ([`binrw::Error`]) from the query packet conversion
pub fn basic_stats<A: ToSocketAddrs>(addr: A) -> Result<response::BasicStat, Error> {
    let socket = get_socket(&addr)?;
    let session_id = get_session_id();
    let request_packet = request::PacketBase {
        packet_type: PacketType::Stat,
        session_id,
        payload: request::BasicStat { challenge_token: handshake(&socket, session_id)? },
    };
    let request: Vec<u8> = request_packet.try_into()?;
    let response_result = send_udp(&socket, &request);
    
    let response = match response_result {
        Ok(res) => { res },
        Err(err) => {
            if err.kind() == std::io::ErrorKind::WouldBlock {
                return basic_stats(addr);
            }
            return Err(err.into());
        }
    };

    let response_packet: response::PacketBase<response::BasicStat> = response.try_into()?;
    Ok(response_packet.payload)
}

/// Gets a struct with the full set of stats collectible by the query protocol
///
/// # Errors
///
/// - **Io** ([`std::io::Error`]) from the [`std::net::UdpSocket`] and writers
/// - **Conversion** ([`std::num::ParseIntError`]) from the query packet conversion
/// - **Conversion** ([`binrw::Error`]) from the query packet conversion
pub fn full_stats<A: ToSocketAddrs>(addr: A) -> Result<response::FullStat, Error> {
    let socket = get_socket(&addr)?;
    let session_id = get_session_id();
    let request_packet = request::PacketBase {
        packet_type: PacketType::Stat,
        session_id,
        payload: request::FullStat { challenge_token: handshake(&socket, session_id)? },
    };
    let request: Vec<u8> = request_packet.try_into()?;
    let response_result = send_udp(&socket, &request);
    let response = match response_result {
        Ok(res) => { res },
        Err(err) => {
            if err.kind() == std::io::ErrorKind::WouldBlock {
                return full_stats(addr);
            } 
            return Err(err.into());
        }
    };
    let response_packet: response::PacketBase<response::FullStat> = response.try_into()?;
    Ok(response_packet.payload)
}

fn handshake(socket: &UdpSocket, session_id: u32) -> Result<u32, Error> {
    let request_packet = request::PacketBase {
        packet_type: PacketType::Handshake,
        session_id,
        payload: request::Handshake
    };
    let request: Vec<u8> = request_packet.try_into()?;
    let response = send_udp(socket, &request)?;
    let response_packet: response::PacketBase<response::Handshake> = response.try_into()?;
    Ok(response_packet.payload.challenge_token)
}

fn send_udp(socket: &UdpSocket, req: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    socket.send(req)?;
    socket.set_read_timeout(Some(std::time::Duration::from_millis(100)))?;

    let mut buf = [0; 65535];
    let len = socket.recv(&mut buf)?;

    Ok(buf[..len].to_vec())
}

fn get_socket<A: ToSocketAddrs>(addr: &A) -> Result<UdpSocket, std::io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(addr)?;
    Ok(socket)
}

fn get_session_id() -> u32 {
    let rand_id: u32 = rand::random();
    rand_id & 0x0F0F_0F0F
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parseinterror: {source}")]
    ParseInt {
        #[from]
        source: std::num::ParseIntError,
    },

    #[error("binrw: {source}")]
    BinRw {
        #[from]
        source: binrw::Error,
    },

    #[error("io: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::{packets::{request, PacketType, response}, send_udp, Error, get_socket};

    const SERVER: &str = "localhost:25565";

    #[test]
    fn handshake() -> Result<(), Error> {
        let socket = get_socket(&SERVER)?;
        crate::handshake(&socket, 0x01020304)?;
        Ok(())
    }

    #[test]
    fn udp() -> Result<(), Error> {
        let socket = get_socket(&SERVER)?;
        let request_packet = request::PacketBase {
            packet_type: PacketType::Handshake,
            session_id: 0x0E030C01,
            payload: request::Handshake
        };
        let request: Vec<u8> = request_packet.try_into()?;
        let response = send_udp(&socket, &request)?;
        let response_packet: Result<response::PacketBase<response::Handshake>, Error> = response.try_into();
        assert!(response_packet.is_ok());
        Ok(())
    }
}
