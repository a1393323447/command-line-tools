use std::io::{BufRead, BufReader, BufWriter, Read, Result, Seek, SeekFrom, StdoutLock, Write};
use structopt::*;

const TABLE: [&str; 32] = [
    "^@ ", "^A ", "^B ", "^C ", "^D ", "^E ", "^F ", "^G ", "^H ", "^I ", "^J ", "^K ", "^L ",
    "^M ", "^N ", "^O ", "^P ", "^Q ", "^R ", "^S ", "^T ", "^U ", "^V ", "^W ", "^X ", "^Y ",
    "^Z ", "^[ ", "^\\ ", "^] ", "^6 ", "^- ",
];

macro_rules! write_fmt_data {
    ($fmt: literal, $writer: tt, $index: expr, $bytes: expr) => {
        $writer.write_fmt(format_args!(
            $fmt,
            $index,
            $bytes[0],
            $bytes[1],
            $bytes[2],
            $bytes[3],
            $bytes[4],
            $bytes[5],
            $bytes[6],
            $bytes[7],
            $bytes[8],
            $bytes[9],
            $bytes[10],
            $bytes[11],
            $bytes[12],
            $bytes[13],
            $bytes[14],
            $bytes[15],
        ))?
    };
}

macro_rules! write_fmt_bin_data {
    ($fmt: literal, $writer: tt, $index: expr, $bytes: expr, $offset: literal) => {
        $writer.write_fmt(format_args!(
            $fmt,
            $index,
            $bytes[0 + $offset],
            $bytes[1 + $offset],
            $bytes[2 + $offset],
            $bytes[3 + $offset],
            $bytes[4 + $offset],
            $bytes[5 + $offset],
            $bytes[6 + $offset],
            $bytes[7 + $offset],
        ))?;
    };
}

macro_rules! write_hex_data {
    ($writer: tt, $index: expr, $bytes: expr) => {
        write_fmt_data!(
            "\x1b[0;32;1m{:08X}\x1b[0m  {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
            $writer,
            $index,
            $bytes
        )
    };
}

macro_rules! write_oct_data {
    ($writer: tt, $index: expr, $bytes: expr) => {
        write_fmt_data!(
            "\x1b[0;32;1m{:08X}\x1b[0m  {:03o} {:03o} {:03o} {:03o} {:03o} {:03o} {:03o} {:03o} {:03o} {:03o} {:03o} {:03o} {:03o} {:03o} {:03o} {:03o}",
            $writer,
            $index,
            $bytes
        )
    };
}

macro_rules! write_bin_data {
    ($writer: tt, $index: expr, $bytes: expr, $offset: literal) => {
        write_fmt_bin_data!(
            "\x1b[0;32;1m{:08X}\x1b[0m  {:08b} {:08b} {:08b} {:08b} {:08b} {:08b} {:08b} {:08b}",
            $writer,
            $index,
            $bytes,
            $offset
        )
    };
}

#[derive(Debug)]
enum Format {
    Bin,
    Oct,
    Hex,
}

impl From<&str> for Format {
    fn from(s: &str) -> Self {
        match s {
            "bin" => Format::Bin,
            "oct" => Format::Oct,
            "hex" => Format::Hex,
            _ => panic!("Invalid format `{}`", s),
        }
    }
}

fn parse_num(s: &str) -> usize {
    s.parse().unwrap()
}

#[derive(Debug)]
enum FileType {
    Stdin,
    File(String),
}

impl From<&str> for FileType {
    fn from(s: &str) -> Self {
        if s == "stdin" {
            FileType::Stdin
        } else {
            FileType::File(s.to_string())
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "dump",
    version = "0.1.0",
    author = "朕与将军解战袍, 1393323447@qq.com",
    about = "Dump date in binary, octonary or hexadecimal format."
)]
pub struct DumpApp {
    /// bin, oct or hex format
    #[structopt(short, long, takes_value = true, parse(from_str = Format::from), default_value = "hex")]
    format: Format,
    /// show only HEAD bytes data
    /// if FILE size less than HEAD bytes, result would be padded with trailing zero [default: all bytes]
    #[structopt(short, long, takes_value = true, parse(from_str = parse_num))]
    head: Option<usize>,
    /// show only TAIL bytes data
    /// if FILE size less than TAIL bytes, result would be padded with trailing zero [default: all bytes]
    #[structopt(short, long, takes_value = true, parse(from_str = parse_num))]
    tail: Option<usize>,

    #[structopt(short, long)]
    #[structopt(
        help = "also show data in ascii char, use `\x1b[0;34;1m.\x1b[0m` to represent invalid ascii code."
    )]
    vis: bool,

    /// FILE to dump
    #[structopt(long = "if", takes_value = true, parse(from_str = FileType::from), default_value = "stdin")]
    ifile: FileType,
}

impl DumpApp {
    pub fn run(&self) -> Result<()> {
        let mut display = false;
        if let Some(len) = self.head {
            self.dump_head_bytes(len)?;
            display = true;
        }

        if let Some(len) = self.tail {
            match self.ifile {
                FileType::Stdin => self.dump_stdin_tail_bytes(len)?,
                FileType::File(ref path) => self.dump_file_tail_bytes(len, path)?,
            }
            display = true;
        }

        if display {
            return Ok(());
        }

        self.dump_all_bytes()?;

        Ok(())
    }

    fn dump_all_bytes(&self) -> Result<()> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut reader: Box<dyn BufRead> = match self.ifile {
            FileType::Stdin => Box::new(BufReader::new(stdin.lock())),
            FileType::File(ref filepath) => {
                Box::new(BufReader::new(std::fs::File::open(filepath)?))
            }
        };
        let mut writer = BufWriter::new(stdout.lock());

        // cache
        let mut index: usize = 0;
        let mut bytes = [0u8; 16];
        // display all
        loop {
            let read_size = reader.read(&mut bytes)?;
            if read_size == 0 {
                break;
            } else {
                bytes[read_size..].iter_mut().for_each(|n| *n = 0);
                if self.vis {
                    self.disply_bytes_vis(&mut index, &bytes, &mut writer)?;
                } else {
                    self.disply_bytes_non_vis(&mut index, &bytes, &mut writer)?;
                }
            }
        }

        Ok(())
    }

    fn dump_head_bytes(&self, len: usize) -> Result<()> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut reader: Box<dyn BufRead> = match self.ifile {
            FileType::Stdin => Box::new(BufReader::new(stdin.lock())),
            FileType::File(ref filepath) => {
                Box::new(BufReader::new(std::fs::File::open(filepath)?))
            }
        };
        let mut writer = BufWriter::new(stdout.lock());

        writer.write_fmt(format_args!(
            "\x1b[0;33;1m                        HEAD {} BYTES\n",
            len
        ))?;

        let mut bytes = vec![0; len + 16 - len % 16];

        let read_size = reader.read(&mut bytes)?;
        bytes[read_size..].iter_mut().for_each(|byte| *byte = 0);

        // padding with 0
        let pad_len = if len % 16 != 0 { 16 - len % 16 } else { 0 };
        for _ in 0..pad_len {
            bytes.push(0);
        }

        // display
        let mut index = 0;
        if self.vis {
            self.disply_bytes_vis(&mut index, &bytes, &mut writer)?;
        } else {
            self.disply_bytes_non_vis(&mut index, &bytes, &mut writer)?;
        }

        writer.write_all(b"\n\n")?;

        Ok(())
    }

    fn dump_file_tail_bytes(&self, len: usize, path: &str) -> Result<()> {
        let mut file = std::fs::File::open(path)?;
        let offset = -(len as i64);
        file.seek(SeekFrom::End(offset))?;
        let stdout = std::io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        let mut reader = BufReader::new(file);

        writer.write_fmt(format_args!(
            "\x1b[0;33;1m                        TAIL {} BYTES\n",
            len
        ))?;

        // reserve padding space
        let cap = if len % 16 != 0 {
            len + 16 - (len % 16)
        } else {
            len
        };
        let mut bytes = vec![0; cap];
        let read_size = reader.read(&mut bytes[0..len])?;
        bytes[read_size..].iter_mut().for_each(|bytes| *bytes = 0);

        // display
        let mut index = 0;
        if self.vis {
            self.disply_bytes_vis(&mut index, &bytes, &mut writer)?;
        } else {
            self.disply_bytes_non_vis(&mut index, &bytes, &mut writer)?;
        }

        writer.write_all(b"\n\n")?;

        Ok(())
    }

    fn dump_stdin_tail_bytes(&self, len: usize) -> Result<()> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut reader = BufReader::new(stdin.lock());
        let mut writer = BufWriter::new(stdout.lock());

        writer.write_fmt(format_args!(
            "\x1b[0;33;1m                        TAIL {} BYTES\n",
            len
        ))?;

        // get tail LEN bytes
        let cap = len + len;
        let mut queue = vec![0; cap + 16 - cap % 16]; // reserve padding space

        let mut start = 0;
        let mut end = len - 1;
        let mut last_read_size = 0;
        loop {
            // queue: [ 0 | 1 ]
            //         len len
            // swap buf 0, buf 1 to be current `bytes` to cache data
            let bytes = &mut queue[start..=end];
            let read_size = reader.read(bytes)?;
            if read_size == 0 {
                break;
            }
            // swap
            start = (start + len) % cap;
            end = (end + len) % cap;
            last_read_size = read_size;
        }

        let (out_start, mut out_end) = if start == 0 {
            // case 1: [x12|34x]
            //         s   e..
            let out_end = end + last_read_size + 1;
            let out_start = out_end - len;
            (out_start, out_end)
        } else {
            // case 2: [34xx|xx12]
            //          ..  s    e
            // copy -> 1. [3434|xx12]
            for cp in 1..(last_read_size + 1) {
                queue[len - cp] = queue[last_read_size - cp];
            }
            //         2. [1234|xx12]
            //                   s2 e
            let reset_bytes_cnt = len - last_read_size;
            let s2 = end + 1 - reset_bytes_cnt;
            for (pos, cp) in (s2..=end).enumerate() {
                queue[pos] = queue[cp];
            }
            (0, len)
        };

        if len % 16 != 0 {
            // padding
            let pad = if len % 16 != 0 { 16 - len % 16 } else { 0 };
            let old_end = out_end;
            out_end = old_end + pad;
            if out_end > cap {
                queue.resize(out_end, 0);
            }
            queue[old_end..out_end]
                .iter_mut()
                .for_each(|byte| *byte = 0);
        }

        let bytes = &mut queue[out_start..out_end];
        let mut index = 0;
        if self.vis {
            self.disply_bytes_vis(&mut index, bytes, &mut writer)?;
        } else {
            self.disply_bytes_non_vis(&mut index, bytes, &mut writer)?;
        }

        writer.write_all(b"\n\n")?;

        Ok(())
    }

    // require bytes.len() % 16 == 0
    fn disply_bytes_non_vis(
        &self,
        index: &mut usize,
        bytes: &[u8],
        writer: &mut BufWriter<StdoutLock>,
    ) -> Result<()> {
        debug_assert!(bytes.len() % 16 == 0, "bytes.len() % 16 != 0");
        let len = bytes.len();
        let mut pos = 0;
        while pos + 16 <= len {
            let out_bytes = &bytes[pos..pos + 16];
            match self.format {
                Format::Bin => {
                    write_bin_data!(writer, *index, out_bytes, 0);
                    writer.write_all(b"\n")?;
                    write_bin_data!(writer, *index + 8, out_bytes, 7);
                    writer.write_all(b"\n")?;
                }
                Format::Oct => {
                    write_oct_data!(writer, *index, out_bytes);
                    writer.write_all(b"\n")?;
                }
                Format::Hex => {
                    write_hex_data!(writer, *index, out_bytes);
                    writer.write_all(b"\n")?;
                }
            }
            *index += 16;
            pos += 16;
        }
        Ok(())
    }

    // require bytes.len() % 16 == 0
    fn disply_bytes_vis(
        &self,
        index: &mut usize,
        bytes: &[u8],
        writer: &mut BufWriter<StdoutLock>,
    ) -> Result<()> {
        debug_assert!(bytes.len() % 16 == 0, "bytes.len() % 16 != 0");
        let len = bytes.len();
        let mut pos = 0;
        while pos + 16 <= len {
            let out_bytes = &bytes[pos..pos + 16];
            match self.format {
                Format::Bin => {
                    write_bin_data!(writer, *index, out_bytes, 0);
                    DumpApp::display_ascii(&out_bytes[0..7], writer)?;
                    writer.write_all(b"\n")?;
                    write_bin_data!(writer, *index + 8, out_bytes, 7);
                    DumpApp::display_ascii(&out_bytes[7..], writer)?;
                    writer.write_all(b"\n")?;
                }
                Format::Oct => {
                    write_oct_data!(writer, *index, out_bytes);
                    DumpApp::display_ascii(out_bytes, writer)?;
                    writer.write_all(b"\n")?;
                }
                Format::Hex => {
                    write_hex_data!(writer, *index, out_bytes);
                    DumpApp::display_ascii(out_bytes, writer)?;
                    writer.write_all(b"\n")?;
                }
            }
            *index += 16;
            pos += 16;
        }
        Ok(())
    }

    #[inline(always)]
    fn display_ascii(bytes: &[u8], writer: &mut BufWriter<StdoutLock>) -> Result<()> {
        let invalid_ascii = b" \x1b[0;34;1m.\x1b[0m ";
        writer.write_all(b"    |")?;
        for byte in bytes {
            if *byte < 127 {
                let c = *byte as char;
                if c.is_control() {
                    writer.write_all(TABLE[c as usize].as_bytes())?;
                } else {
                    writer.write_fmt(format_args!(" {} ", c))?;
                }
            } else {
                writer.write_all(invalid_ascii)?;
            }
        }

        Ok(())
    }
}
