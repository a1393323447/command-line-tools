// 报文格式参考资料(相关 RFC ):
// ICMPv4: https://www.rfc-editor.org/pdfrfc/rfc792.txt.pdf
// ICMPv6: https://www.rfc-editor.org/pdfrfc/rfc4443.txt.pdf

mod echo;
mod error;
mod timestamp;

pub use echo::{Echo, EchoReply, EchoRequest};
pub use error::{DecodeError, DecodeResult};
pub use timestamp::{Timestamp, TimestampMessage, TimestampReply, TimestampRequest};

pub struct IcmpV4;
pub struct IcmpV6;

pub const HEADER_SIZE: usize = 8;

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
