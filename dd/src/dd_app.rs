use std::str::FromStr;

use crate::after_help::*;
use clap::{App, Arg};

#[derive(Debug)]
enum FileType {
    Stdin,
    Stdout,
    File(String),
}

#[derive(Debug)]
enum Flag {
    None,
    Append,
    Direct,
    Directory,
    DataSync,
    Sync,
    FullBlock,
    Nonblock,
    NoAccessTime,
    NoCache,
    NoCTTY,
    NoFollow,
    CountBytes,
    SkipBytes,
    SeekBytes,
}

impl Default for Flag {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug)]
enum Conv {
    None,
    Ascii,
    Ebcdic,
    Ibm,
    Block,
    Unblock,
    LowerCase,
    UpperCase,
    Sparse,
    SwapByte,
    Sync,
    Excl,
    Nocreat,
    Notrunc,
    Noerror,
    FileDataSync,
    FileSync,
}

impl Default for Conv {
    fn default() -> Self {
        Self::None
    }
}

impl From<&str> for Conv {
    fn from(s: &str) -> Self {
        use Conv::*;
        match s {
            "ascii" => Ascii,
            "ebcdic" => Ebcdic,
            "ibm" => Ibm,
            "block" => Block,
            "unblock" => Unblock,
            "lcase" => LowerCase,
            "ucase" => UpperCase,
            "sparse" => Sparse,
            "swab" => SwapByte,
            "sync" => Sync,
            "excl" => Excl,
            "nocreat" => Nocreat,
            "notrunc" => Notrunc,
            "noerror" => Noerror,
            "fdatasync" => FileDataSync,
            "fsync" => FileSync,
            _ => panic!("Invaild conv value {}", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
struct BlockSize(usize);

impl From<&str> for BlockSize {
    fn from(s: &str) -> Self {
        if s.is_empty() {
            return BlockSize(512); // default value
        }

        let mut count = String::new();
        let mut n = 0;
        for c in s.chars() {
            if c.is_numeric() {
                count.push(c);
                n += 1;
            } else {
                break;
            }
        }
        let count = match count.parse::<usize>() {
            Err(err) => panic!("{}", err),
            Ok(0) => panic!("Invalid BYTES value `0`"),
            Ok(c) => c,
        };

        let unit: String = s.chars().skip(n).collect();
        let scale: usize = match unit.as_str() {
            "" | "c" => 1,
            "w" => 2,
            "b" => 512,
            "kB" => 1000,
            "K" | "KiB" => 1024,
            "MB" => 1000 * 1000,
            "M" | "xM" | "MiB" => 1024 * 1024,
            "GB" => 1000 * 1000 * 1000,
            "G" | "GiB" => 1024 * 1024 * 1024,
            _ => panic!("Unrecognized unit `{}`", unit),
        };
        BlockSize(count * scale)
    }
}

#[derive(Debug)]
pub struct DDApp {
    ibs: BlockSize,
    obs: BlockSize,
    cbs: BlockSize,
    count: Option<usize>,
    seek: usize,
    skip: usize,
    ifile: FileType,
    ofile: FileType,
    iflag: Flag,
    oflag: Flag,
    conv: Conv,
}

impl DDApp {
    pub fn new() -> Self {
        DDApp {
            ibs: Default::default(),
            obs: Default::default(),
            cbs: Default::default(),
            count: Default::default(),
            seek: Default::default(),
            skip: Default::default(),
            ifile: FileType::Stdin,
            ofile: FileType::Stdout,
            iflag: Default::default(),
            oflag: Default::default(),
            conv: Default::default(),
        }
    }

    pub fn get_args(&mut self) {
        let matches = App::new("dd")
            .arg(Arg::new("BYTES").long("bs").takes_value(true).help(concat!(
                "read and write up to BYTES bytes at a time (default: 512);\n",
                "overrides ibs and obs"
            )))
            .arg(
                Arg::new("CBYTES")
                    .long("cbs")
                    .takes_value(true)
                    .help("convert CBYTES bytes at a time"),
            )
            .arg(
                Arg::new("CONVS")
                    .long("conv")
                    .takes_value(true)
                    .help("convert the file as per the comma separated symbol list"),
            )
            .arg(
                Arg::new("N-COUNT")
                    .long("count")
                    .takes_value(true)
                    .help("copy only N-COUNT input blocks"),
            )
            .arg(
                Arg::new("IBYTES")
                    .long("ibs")
                    .takes_value(true)
                    .help("read up to BYTES bytes at a time (default: 512)"),
            )
            .arg(
                Arg::new("IFLIE")
                    .long("if")
                    .takes_value(true)
                    .help("read from IFILE instead of stdin"),
            )
            .arg(
                Arg::new("IFLAGS")
                    .long("iflag")
                    .takes_value(true)
                    .help("read as per the comma separated symbol list"),
            )
            .arg(
                Arg::new("OBYTES")
                    .long("obs")
                    .takes_value(true)
                    .help("write BYTES bytes at a time (default: 512)"),
            )
            .arg(
                Arg::new("OFILE")
                    .long("of")
                    .takes_value(true)
                    .help("write to OFILE instead of stdout"),
            )
            .arg(
                Arg::new("OFLAGS")
                    .long("oflag")
                    .takes_value(true)
                    .help("write as per the comma separated symbol list"),
            )
            .arg(
                Arg::new("N-SEEK")
                    .long("seek")
                    .takes_value(true)
                    .help("skip N-SEEK obs-sized blocks at start of output"),
            )
            .arg(
                Arg::new("N-SKIP")
                    .long("skip")
                    .takes_value(true)
                    .help("skip N-SKIP obs-sized blocks at start of input"),
            )
            .arg(
                Arg::new("LEVEL")
                    .long("status")
                    .takes_value(true)
                    .help(concat!(
                        "The LEVEL of information to print to stderr\n",
                        "'none' suppresses everything but error messages\n",
                        "'noxfer' suppresses the final transfer statistics\n",
                        "'progress' shows periodic transfer statistics"
                    )),
            )
            .name("dd")
            .version("0.1.0")
            .author("朕与将军解战袍, 1393323447@qq.com")
            .about("convert and copy a file")
            .long_about("Copy a file, converting and formatting according to the operands.")
            .after_help(AFTER_HELP_STR)
            .get_matches();

        self.ibs = matches.value_of("IBYTES").unwrap_or("512").into();
        self.obs = matches.value_of("OBYTES").unwrap_or("512").into();
        if let Some(s) = matches.value_of("BYTES") {
            let bs = s.into();
            self.ibs = bs;
            self.obs = bs;
        }
        self.cbs = matches.value_of("CBYTES").unwrap_or("512").into();
    }
}

#[cfg(test)]
mod test {
    use crate::dd_app::*;

    #[test]
    fn parse_block_size() {
        assert_eq!(BlockSize(512), "".into());
        assert_eq!(BlockSize(2), "2".into());
        assert_eq!(BlockSize(20), "20".into());
        assert_eq!(BlockSize(233), "233c".into());
        assert_eq!(BlockSize(256), "128w".into());
        assert_eq!(BlockSize(5120), "10b".into());
        assert_eq!(BlockSize(10 * 1000), "10kB".into());
        assert_eq!(BlockSize(12 * 1024), "12K".into());
        assert_eq!(BlockSize(12 * 1024), "12KiB".into());
    }
}
