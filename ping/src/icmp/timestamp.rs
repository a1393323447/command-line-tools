#![allow(unused)]

// ICMPv4
// Timestamp or Timestamp Reply Message
//  |       0       |       1       |       2       |       3       |
//  |0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7|
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |     Type      |      Code     |           Checksum            |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |           Identifier          |        Sequence Number        |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |                        Originate Timestamp                    |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |                         Receive Timestamp                     |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |                       Transmit Timestamp                      |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  IP Fields:
//  Addresses
//      The address of the source in a timestamp message will be the
//      destination of the timestamp reply message. To form a timestamp
//      reply message, the source and destination addresses are simply
//      reversed, the type code changed to 14, and the checksum
//      recomputed.
//  IP Fields:
//  Type
//      13 for timestamp message;
//      14 for timestamp reply message.
//  Code
//      0
//  Checksum
//      The checksum is the 16-bit ones’s complement of the one’s
//      complement sum of the ICMP message starting with the ICMP Type.
//      For computing the checksum , the checksum field should be zero.
//      This checksum may be replaced in the future.
//  Identifier
//      If code = 0, an identifier to aid in matching timestamp and
//      replies, may be zero.
//  Sequence Number
//      If code = 0, a sequence number to aid in matching timestamp and
//      replies, may be zero.
//  Description
//      The data received (a timestamp) in the message is returned in the
//      reply together with an additional timestamp. The timestamp is 32
//      bits of milliseconds since midnight UT. One use of these
//      timestamps is described by Mills [5].
//
//      The Originate Timestamp is the time the sender last touched the
//      message before sending it, the Receive Timestamp is the time the
//      echoer first touched it on receipt, and the Transmit Timestamp is
//      the time the echoer last touched the message on sending it.
//
//      If the time is not available in miliseconds or cannot be provided
//      with respect to midnight UT then any time can be inserted in a
//      timestamp provided the high order bit of the timestamp is also set
//      to indicate this non-standard value.
//      The identifier and sequence number may be used by the echo sender
//      to aid in matching the replies with the requests. For example,
//      the identifier might be used like a port in TCP or UDP to identify
//      a session, and the sequence number might be incremented on each
//      request sent. The destination returns these same values in the
//      reply.
//
//      Code 0 may be received from a gateway or a host.

use super::{write_checksum, DecodeError, DecodeResult, IcmpV4, HEADER_SIZE};

use std::{
    fmt::Display,
    ops::{Add, Mul, Sub},
    time::{Duration, SystemTime},
};

pub trait TimestampMessage {
    const REQUEST_TYPE: u8;
    const REQUEST_CODE: u8;
    const REPLY_TYPE: u8;
    const REPLY_CODE: u8;
}

impl TimestampMessage for IcmpV4 {
    const REQUEST_TYPE: u8 = 13;
    const REQUEST_CODE: u8 = 0;
    const REPLY_TYPE: u8 = 14;
    const REPLY_CODE: u8 = 0;
}

pub struct TimestampRequest {
    pub ident: u16,
    pub seq_cnt: u16,
    pub orig_timestamp: Timestamp,
    pub recv_timestamp: Timestamp,
    pub tran_timestamp: Timestamp,
}

impl TimestampRequest {
    pub fn new(ident: u16, seq_cnt: u16) -> TimestampRequest {
        TimestampRequest {
            ident,
            seq_cnt,
            orig_timestamp: Timestamp::now(),
            recv_timestamp: Timestamp::now(),
            tran_timestamp: Timestamp::now(),
        }
    }
    pub fn encode<P>(&self, buffer: &mut [u8])
    where
        P: TimestampMessage,
    {
        buffer[0] = P::REQUEST_TYPE;
        buffer[1] = P::REQUEST_CODE;

        buffer[4..6].clone_from_slice(&self.ident.to_be_bytes());
        buffer[6..8].clone_from_slice(&self.seq_cnt.to_be_bytes());
        buffer[8..12].clone_from_slice(&self.orig_timestamp.to_be_bytes());
        buffer[12..16].clone_from_slice(&self.recv_timestamp.to_be_bytes());
        buffer[16..20].clone_from_slice(&self.tran_timestamp.to_be_bytes());

        write_checksum(buffer);
    }
}

pub struct TimestampReply {
    pub ident: u16,
    pub seq_cnt: u16,
    pub orig_timestamp: Timestamp,
    pub recv_timestamp: Timestamp,
    pub tran_timestamp: Timestamp,
}

impl TimestampReply {
    pub fn decode<P: TimestampMessage>(buffer: &[u8]) -> DecodeResult<TimestampReply> {
        if buffer.as_ref().len() < HEADER_SIZE {
            return Err(DecodeError::InvalidSize);
        }

        let type_ = buffer[0];
        let code = buffer[1];
        if type_ != P::REPLY_TYPE && code != P::REPLY_CODE {
            return Err(DecodeError::InvalidPacket);
        }

        let ident = (u16::from(buffer[4]) << 8) + u16::from(buffer[5]);
        let seq_cnt = (u16::from(buffer[6]) << 8) + u16::from(buffer[7]);

        let orig_timestamp = Timestamp::from_bytes(&buffer[8..12]);
        let recv_timestamp = Timestamp::from_bytes(&buffer[12..16]);
        let tran_timestamp = Timestamp::from_bytes(&buffer[16..20]);

        Ok(TimestampReply {
            ident,
            seq_cnt,
            orig_timestamp,
            recv_timestamp,
            tran_timestamp,
        })
    }
}

#[derive(Debug)]
pub struct Timestamp(u32);

impl Timestamp {
    pub fn now() -> Timestamp {
        let now = SystemTime::now();
        let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();

        let t: Timestamp = duration.into();
        Timestamp(t.0 - 2182158336)
    }

    fn to_be_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Timestamp {
        assert!(bytes.len() == 4);

        let mut data = [0u8; 4];
        data.clone_from_slice(bytes);

        let stamp = u32::from_le_bytes(data);

        Timestamp(stamp)
    }
}

impl From<Duration> for Timestamp {
    fn from(delta: Duration) -> Self {
        Timestamp(delta.as_millis() as u32)
    }
}

impl From<Timestamp> for Duration {
    fn from(stamp: Timestamp) -> Self {
        Duration::from_millis(stamp.0 as u64)
    }
}

impl Add for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: Self) -> Self::Output {
        Timestamp(self.0.overflowing_add(rhs.0).0)
    }
}

impl Sub for Timestamp {
    type Output = Timestamp;

    fn sub(self, rhs: Self) -> Self::Output {
        Timestamp(self.0.overflowing_sub(rhs.0).0)
    }
}

impl Mul<u32> for Timestamp {
    type Output = Timestamp;

    fn mul(self, rhs: u32) -> Self::Output {
        Timestamp(self.0.overflowing_mul(rhs).0)
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ms", self.0)
    }
}

#[test]
fn test_timestamp() {
    let t = Timestamp::now();
    println!("{}", t);
}
