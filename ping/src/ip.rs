// IPv4: https://www.rfc-editor.org/pdfrfc/rfc791.txt.pdf
// IPv6: https://www.rfc-editor.org/pdfrfc/rfc1883.txt.pdf

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("too small header")]
    TooSmallHeader,
    #[error("invalid header size")]
    InvalidHeaderSize,
    #[error("invalid version")]
    InvalidVersion,
    #[error("unknown protocol")]
    UnknownProtocol,
}

const MINIMUM_PACKET_SIZE: usize = 20;

#[derive(Debug, PartialEq)]
pub enum IpV4Protocol {
    Icmp,
}

impl IpV4Protocol {
    fn decode(data: u8) -> Option<Self> {
        match data {
            1 => Some(IpV4Protocol::Icmp),
            _ => None,
        }
    }
}

pub struct IpV4Packet<'a> {
    pub protocol: IpV4Protocol,
    pub ttl: u8,
    pub data: &'a [u8],
}

impl<'a> IpV4Packet<'a> {
    pub fn decode(data: &'a [u8]) -> Result<Self, Error> {
        if data.len() < MINIMUM_PACKET_SIZE {
            return Err(Error::TooSmallHeader);
        }
        let byte0 = data[0];
        let version = (byte0 & 0xf0) >> 4;
        let header_size = 4 * ((byte0 & 0x0f) as usize);

        if version != 4 {
            return Err(Error::InvalidVersion);
        }

        if data.len() < header_size {
            return Err(Error::InvalidHeaderSize);
        }

        let protocol = match IpV4Protocol::decode(data[9]) {
            Some(protocol) => protocol,
            None => return Err(Error::UnknownProtocol),
        };

        let ttl = data[8];

        Ok(Self {
            protocol,
            ttl,
            data: &data[header_size..],
        })
    }
}