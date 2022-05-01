// 报文格式参考资料(相关 RFC ): 
// ICMPv4: https://www.rfc-editor.org/pdfrfc/rfc792.txt.pdf
// ICMPv6: https://www.rfc-editor.org/pdfrfc/rfc1885.txt.pdf

// 摘要:

// Echo or Echo Reply Message
//  |       0       |       1       |       2       |       3       |
//  |0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7|
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |     Type      |        Code       |         Checksum          |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |             Identifier            |      Sequence Number      |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |   Data   ...
//  +-+-+-+-+-
//  IP Fields:
//  Addresses
//      The address of the source in an echo message will be the
//      destination of the echo reply message. To form an echo reply
//      message, the source and destination addresses are simply reversed,
//      the type code changed to 0, and the checksum recomputed.
//  IP Fields:
//  Type
//      8 for echo message;
//      0 for echo reply message.
//  Code
//      0
//  Checksum
//      The checksum is the 16-bit ones’s complement of the one’s
//      complement sum of the ICMP message starting with the ICMP Type.
//      For computing the checksum , the checksum field should be zero.
//      If the total length is odd, the received data is padded with one
//      octet of zeros for computing the checksum. This checksum may be
//      replaced in the future.
//  Identifier
//      If code = 0, an identifier to aid in matching echos and replies,
//      may be zero.
//  Sequence Number
//      If code = 0, a sequence number to aid in matching echos and
//      replies, may be zero.
//  Description
//      The data received in the echo message must be returned in the echo
//      reply message.
//      The identifier and sequence number may be used by the echo sender
//      to aid in matching the replies with the echo requests. For
//      example, the identifier might be used like a port in TCP or UDP to
//      identify a session, and the sequence number might be incremented
//      on each echo request sent. The echoer returns these same values
//      in the echo reply.
//      Code 0 may be received from a gateway or a host.

// 4. ICMPv6 Informational Messages
// 4.1 Echo Request Message
//  |       0       |       1       |       2       |       3       |
//  |0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7|
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |     Type      |        Code       |         Checksum          |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |             Identifier            |      Sequence Number      |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |   Data   ...
//  +-+-+-+-+-
//  IPv6 Fields:
//      Destination Address
//                  Any legal IPv6 address.
//  ICMPv6 Fields:
//  Type            128
//  Code            0
//  Identifier      An identifier to aid in matching Echo Replies
//                  to this Echo Request. May be zero.
//  Sequence Number
//                  A sequence number to aid in matching Echo Replies
//                  to this Echo Request. May be zero.
//  Data            Zero or more octets of arbitrary data.
// 
//  Description
// 
//  Every node MUST implement an ICMPv6 Echo responder function that
//  receives Echo Requests and sends corresponding Echo Replies. A node
//  SHOULD also implement an application-layer interface for sending Echo
//  Requests and receiving Echo Replies, for diagnostic purposes.
//  Upper layer notification
//  A node receiving this ICMPv6 message MAY notify the upper-layer
//  protocol.

// 4.2 Echo Reply Message
//  |       0       |       1       |       2       |       3       |
//  |0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7|
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |     Type      |        Code       |         Checksum          |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |             Identifier            |      Sequence Number      |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |   Data   ...
//  +-+-+-+-+-
//  IPv6 Fields:
//      Destination Address
//                  Copied from the Source Address field of the invoking
//  Echo Request packet.
//  ICMPv6 Fields:
//  Type            129
//  Code            0
//  Identifier      The identifier from the invoking Echo Request message.
//  Sequence        The sequence number from the invoking Echo Request
//                  Number message.
//  Data            The data from the invoking Echo Request message.
//  Description
//
//  Every node MUST implement an ICMPv6 Echo responder function that
//  receives Echo Requests and sends corresponding Echo Replies. A node
//  SHOULD also implement an application-layer interface for sending Echo
//  Requests and receiving Echo Replies, for diagnostic purposes.
//
//  The source address of an Echo Reply sent in response to a unicast
//  Echo Request message MUST be the same as the destination address of
//  that Echo Request message.
//
//  An Echo Reply SHOULD be sent in response to an Echo Request message
//  sent to an IPv6 multicast address. The source address of the reply
//  MUST be a unicast address belonging to the interface on which the
//  multicast Echo Request message was received.
//  The data received in the ICMPv6 Echo Request message MUST be returned
//  entirely and unmodified in the ICMPv6 Echo Reply message, unless the
//  Echo Reply would exceed the MTU of the path back to the Echo
//  requester, in which case the data is truncated to fit that path MTU.
//
//  Upper layer notification
// 
//  Echo Reply messages MUST be passed to the ICMPv6 user interface,
//  unless the corresponding Echo Request originated in the IP layer.

use thiserror::Error;
use std::io::Write;

pub const HEADER_SIZE: usize = 8;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid size")]
    InvalidSize,
    #[error("invalid packet")]
    InvalidPacket,
}

pub struct IcmpV4;
pub struct IcmpV6;

pub trait Proto {
    const ECHO_REQUEST_TYPE: u8;
    const ECHO_REQUEST_CODE: u8;
    const ECHO_REPLY_TYPE: u8;
    const ECHO_REPLY_CODE: u8;
}

impl Proto for IcmpV4 {
    const ECHO_REQUEST_TYPE: u8 = 8;
    const ECHO_REQUEST_CODE: u8 = 0;
    const ECHO_REPLY_TYPE: u8 = 0;
    const ECHO_REPLY_CODE: u8 = 0;
}

impl Proto for IcmpV6 {
    const ECHO_REQUEST_TYPE: u8 = 128;
    const ECHO_REQUEST_CODE: u8 = 0;
    const ECHO_REPLY_TYPE: u8 = 129;
    const ECHO_REPLY_CODE: u8 = 0;
}

pub struct EchoRequest<'a> {
    pub ident: u16,
    pub seq_cnt: u16,
    pub payload: &'a [u8],
}

impl<'a> EchoRequest<'a> {
    pub fn encode<P: Proto>(&self, buffer: &mut [u8]) -> Result<(), Error> {
        buffer[0] = P::ECHO_REQUEST_TYPE;
        buffer[1] = P::ECHO_REQUEST_CODE;

        buffer[4] = (self.ident >> 8) as u8;
        buffer[5] = self.ident as u8;
        buffer[6] = (self.seq_cnt >> 8) as u8;
        buffer[7] = self.seq_cnt as u8;

        if (&mut buffer[8..]).write(self.payload).is_err() {
            return Err(Error::InvalidSize)
        }

        write_checksum(buffer);
        Ok(())
    }
}

pub struct EchoReply<'a> {
    pub ident: u16,
    pub seq_cnt: u16,
    pub payload: &'a [u8]
}

impl<'a> EchoReply<'a> {
    pub fn decode<P: Proto>(buffer: &'a [u8]) -> Result<Self, Error> {
        if buffer.as_ref().len() < HEADER_SIZE {
            return Err(Error::InvalidSize)
        }

        let type_ = buffer[0];
        let code = buffer[1];
        if type_ != P::ECHO_REPLY_TYPE && code != P::ECHO_REPLY_CODE {
            return Err(Error::InvalidPacket)
        }

        let ident = (u16::from(buffer[4]) << 8) + u16::from(buffer[5]);
        let seq_cnt = (u16::from(buffer[6]) << 8) + u16::from(buffer[7]);

        let payload = &buffer[HEADER_SIZE..];

        Ok(EchoReply {
            ident, seq_cnt, payload
        })
    }
}

/// 校验和
fn get_checksum(buffer: &[u8]) -> u16 {
    // 1. 将校验和字段置为 0
    let mut sum = 0u32;

    // 2. 将每两个字节（16位）相加（二进制求和）直到最后得出结果, 若出现最后还剩一个字节继续与前面结果相加
    for word in buffer.chunks(2) {
        let mut part = u16::from(word[0]) << 8;
        if word.len() > 1 {
            part += u16::from(word[1]);
        }
        sum = sum.wrapping_add(u32::from(part));
    }

    // 3. 将和的高 16 位与低 16 位相加，直到高16位为 0 为止
    while (sum >> 16) > 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    // 4. 将最后的结果（二进制）取反
    !sum as u16
}

fn write_checksum(buffer: &mut [u8]) {
    let sum = get_checksum(buffer);
    buffer[2] = (sum >> 8) as u8;
    buffer[3] = (sum & 0xff) as u8;
}
