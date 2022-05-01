use clap::{App, Arg, ValueHint};

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Result};

use std::io::Write;

const TABLE: [&str; 32] = [
    "^@", "^A", "^B", "^C", "^D", "^E", "^F", "^G", "^H", "\t", "\x0a", "^K", "^L", "\x0d", "^N",
    "^O", "^P", "^Q", "^R", "^S", "^T", "^U", "^V", "^W", "^X", "^Y", "^Z", "^[", "^\\", "^]",
    "^6", "^-",
];

#[derive(Debug)]
enum FileType {
    Stdin,
    File(String),
}

#[derive(Debug)]
enum LineNumberStrategy {
    None,
    Normal,
    NonBlank,
}

impl Default for LineNumberStrategy {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ShowStrategy {
    squeeze: bool,
    show_ends: bool,
    show_tabs: bool,
    show_nonprinting: bool,
}

#[derive(Debug, Default)]
struct Config {
    line_number_strategy: LineNumberStrategy,
    show_strategy: ShowStrategy,
}

#[derive(Debug)]
pub struct CatApp {
    config: Config,
    file_type: FileType,
}

impl CatApp {
    pub fn new() -> Self {
        CatApp {
            config: Default::default(),
            file_type: FileType::Stdin,
        }
    }

    pub fn get_args(&mut self) {
        let matches = App::new("cat")
            .arg(
                Arg::new("show-all")
                    .short('A')
                    .long("show-all")
                    .help("equivalent to -vET"),
            )
            .arg(
                Arg::new("number-nonblank")
                    .short('b')
                    .long("number-nonblank")
                    .help("number nonempty output lines, overrides -n"),
            )
            .arg(Arg::new("e").short('e').help("equivalent to -vE"))
            .arg(
                Arg::new("show-ends")
                    .short('E')
                    .long("show-ends")
                    .help("display $ at end of each line"),
            )
            .arg(
                Arg::new("number")
                    .short('n')
                    .long("number")
                    .help("number all output lines"),
            )
            .arg(
                Arg::new("squeeze-nonblank")
                    .short('s')
                    .long("squeeze-nonblank")
                    .help("suppress repeated empty output lines"),
            )
            .arg(Arg::new("t").short('t').help("equivalent to -vT"))
            .arg(
                Arg::new("show-tabs")
                    .short('T')
                    .long("show-tabs")
                    .help("display TAB characters as ^I"),
            )
            .arg(
                Arg::new("show-nonprinting")
                    .short('v')
                    .long("show-nonprinting")
                    .help("use ^ and M- notation, except for LFD and TAB"),
            )
            .arg(
                Arg::new("FILE")
                    .takes_value(true)
                    .value_hint(ValueHint::FilePath)
                    .help(concat!(
                        "FILE(s) would be concatenate to standard output.\n",
                        "With no FILE, or when FILE is -, read standard input."
                    )),
            )
            .about("concatenate files and print on the standard output")
            .long_about(concat!(
                "\nConcatenate FILE(s) to standard output.\n",
                "With no FILE, or when FILE is -, read standard input."
            ))
            .version("0.1.0")
            .author("朕与将军解战袍, 1393323447@qq.com")
            .get_matches();

        if matches.is_present("number") {
            self.config.line_number_strategy = LineNumberStrategy::Normal;
        } else if matches.is_present("number-nonblank") {
            self.config.line_number_strategy = LineNumberStrategy::NonBlank;
        }

        self.config.show_strategy.squeeze = matches.is_present("squeeze-nonblank");
        self.config.show_strategy.show_ends = matches.is_present("show-ends");
        self.config.show_strategy.show_tabs = matches.is_present("show-tabs");
        self.config.show_strategy.show_nonprinting = matches.is_present("show-nonprinting");

        let filepath = matches.value_of("FILE").unwrap_or("-").to_string();
        self.file_type = if filepath == "-" {
            FileType::Stdin
        } else {
            FileType::File(filepath)
        };

        if matches.is_present("show-all") {
            self.config.show_strategy.show_nonprinting = true; // -v
            self.config.show_strategy.show_ends = true; // -E
            self.config.show_strategy.show_tabs = true; // -T
        }

        if matches.is_present("e") {
            self.config.show_strategy.show_nonprinting = true; // -v
            self.config.show_strategy.show_ends = true; // -E
        }

        if matches.is_present("t") {
            self.config.show_strategy.show_nonprinting = true; // -v
            self.config.show_strategy.show_tabs = true; // -T
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let stdin = io::stdin();
        let stdout = std::io::stdout();

        let mut reader: Box<dyn BufRead> = match self.file_type {
            FileType::Stdin => Box::new(BufReader::new(stdin.lock())),
            FileType::File(ref filepath) => Box::new(BufReader::new(File::open(filepath)?)),
        };
        let mut writer = BufWriter::new(stdout);

        let mut line = String::with_capacity(1024);
        let mut line_num = 1;

        let ShowStrategy {
            squeeze,
            show_ends,
            show_tabs,
            show_nonprinting,
        } = self.config.show_strategy;

        loop {
            match reader.read_line(&mut line) {
                Ok(0) => {
                    break;
                }
                Ok(_) => { /* Do nothing */ }
                Err(err) => {
                    return Err(err);
                }
            }

            // check if line is blank
            let mut blank_line = true;
            for (cnt, c) in line.chars().skip_while(|c| *c == ' ').enumerate() {
                if cnt >= 2 || !(c == '\r' && c == '\x0a') {
                    blank_line = false;
                    break;
                }
            }

            // out put
            if !blank_line || !squeeze {
                match self.config.line_number_strategy {
                    LineNumberStrategy::None => { /* Do nothing */ }
                    LineNumberStrategy::Normal => {
                        writer.write_fmt(format_args!("\x1b[0;32;1m{}\x1b[0m ", line_num))?;
                        line_num += 1;
                    }
                    LineNumberStrategy::NonBlank => {
                        if blank_line {
                            /* Do nothing */
                        } else {
                            writer.write_fmt(format_args!("\x1b[0;32;1m{}\x1b[0m ", line_num))?;
                            line_num += 1;
                        }
                    }
                }

                for c in line.chars() {
                    if c == '\r' {
                        continue;
                    }
                    if c == '\t' && show_tabs {
                        writer.write_all(b"\x1b[0;33;1m^I\x1b[0m")?;
                        continue;
                    }
                    if c == '\n' && show_ends {
                        writer.write_all(b"\x1b[0;34;1m$\x1b[0m\n")?;
                        continue;
                    }
                    if c.is_control() && show_nonprinting && c != '\t' {
                        writer.write_all(b"\x1b[0;35;4m")?;
                        writer.write_all(TABLE[c as usize].as_bytes())?;
                        writer.write_all(b"\x1b[0m")?;
                        continue;
                    }
                    writer.write_fmt(format_args!("{}", c))?;
                }
            }

            line.clear();
        }

        if show_ends {
            writer.write_all(b"\x1b[0;34;1m$\x1b[0m")?;
        }

        Ok(())
    }
}
