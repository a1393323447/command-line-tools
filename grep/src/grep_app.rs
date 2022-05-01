use clap::{App, Arg};
use crossbeam::channel::{self, Sender};
use regex::Regex;

use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Result, StdoutLock, Write};
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "unix")]
use std::os::unix::fs::MetadataExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;

const K: u64 = 1024;

#[derive(Debug)]
pub struct GrepApp {
    pattern: Regex,
    files: Option<Vec<String>>,
    dirs: Option<Vec<String>>,
    recursive: bool,
    thread_num: usize,
}

impl GrepApp {
    pub fn new() -> Self {
        GrepApp {
            pattern: Regex::new("a").unwrap(),
            files: None,
            dirs: None,
            recursive: false,
            thread_num: 4,
        }
    }

    pub fn get_args(&mut self) {
        let matches = App::new("grep")
            .arg(Arg::new("PATTERN").takes_value(true))
            .arg(
                Arg::new("FILE")
                    .long("file")
                    .short('f')
                    .help("FILE(s) to match")
                    .takes_value(true)
                    .multiple_values(true),
            )
            .arg(
                Arg::new("DIR")
                    .long("dir")
                    .short('d')
                    .help("DIR(s) to match")
                    .takes_value(true)
                    .multiple_values(true),
            )
            .arg(
                Arg::new("recursive")
                    .short('r')
                    .help("match recursively")
                    .conflicts_with("FILE"),
            )
            .arg(
                Arg::new("N-THREAD")
                    .long("n-thread")
                    .help("use N-THREAD to match, default value is 4")
                    .takes_value(true)
                    .default_value("4"),
            )
            .version("0.1.0")
            .author("朕与将军解战袍, 1393323447@qq.com")
            .about("match content in file or directory")
            .get_matches();

        self.pattern = Regex::new(
            matches
                .value_of("PATTERN")
                .expect("Please provide a PATTERN to match"),
        )
        .expect("Invalid pattern: ");
        self.files = matches
            .values_of("FILE")
            .map(|values| values.map(|s| s.to_string()).collect());
        self.dirs = matches
            .values_of("DIR")
            .map(|values| values.map(|s| s.to_string()).collect());
        if self.files.is_none() && self.dirs.is_none() {
            panic!("Please specify FILE(s) or DIR(s) to match");
        }
        self.recursive = matches.is_present("recursive");
        self.thread_num = matches.value_of("N-THREAD").unwrap_or("4").parse().unwrap();
    }

    pub fn run(&self) -> Result<()> {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.thread_num)
            .build_global()
            .unwrap();

        let stdout = std::io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        let (sender, receiver) = channel::unbounded();

        self.search_and_match_files(sender.clone(), &mut writer)?;
        drop(sender);

        while let Ok(buf) = receiver.recv() {
            writer.write_all(&buf)?;
        }

        Ok(())
    }

    fn search_and_match_files(
        &self,
        sender: Sender<Vec<u8>>,
        writer: &mut BufWriter<StdoutLock>,
    ) -> Result<()> {
        let mut path = PathBuf::new();
        if let Some(ref files) = self.files {
            for filepath in files {
                path.push(filepath);
                self.start_match(&path, sender.clone(), writer)?;
                path.clear();
            }
        }

        if let Some(ref dirs) = self.dirs {
            if self.recursive {
                for dir in dirs {
                    self.search_files_recursively(dir, sender.clone(), writer)?;
                }
            } else {
                for dir in dirs {
                    self.search_files_nonreursively(dir, sender.clone(), writer)?;
                }
            }
        }

        Ok(())
    }

    fn search_files_recursively(
        &self,
        dir: &str,
        sender: Sender<Vec<u8>>,
        writer: &mut BufWriter<StdoutLock>,
    ) -> Result<()> {
        let mut dirs = vec![dir.into()];
        while !dirs.is_empty() {
            let dir = dirs.pop().unwrap();
            let entries = fs::read_dir(dir)?;
            for entry in entries {
                let entry = entry?;
                let file_type = entry.file_type()?;
                if file_type.is_dir() {
                    dirs.push(entry.path());
                } else if file_type.is_file() {
                    self.start_match(&entry.path(), sender.clone(), writer)?;
                }
            }
        }

        Ok(())
    }

    fn search_files_nonreursively(
        &self,
        dir: &str,
        sender: Sender<Vec<u8>>,
        writer: &mut BufWriter<StdoutLock>,
    ) -> Result<()> {
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_file() {
                self.start_match(&entry.path(), sender.clone(), writer)?;
            }
        }

        Ok(())
    }

    fn start_match(
        &self,
        filepath: &PathBuf,
        sender: Sender<Vec<u8>>,
        writer: &mut BufWriter<StdoutLock>,
    ) -> Result<()> {
        let meta = fs::metadata(filepath)?;
        let file_size = meta.file_size();
        if file_size >= 5 * K {
            let p = self.pattern.clone();
            let f = filepath.clone();
            rayon::spawn(move || GrepApp::parallel_match_content(p, f, sender));
        } else {
            self.match_content(filepath, writer)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn parallel_match_content(pattern: Regex, filepath: PathBuf, sender: Sender<Vec<u8>>) {
        GrepApp::parallel_match_content_wrap(pattern, filepath, sender)
            .expect("Failed to write buffer ");
    }

    fn parallel_match_content_wrap(
        pattern: Regex,
        filepath: PathBuf,
        sender: Sender<Vec<u8>>,
    ) -> Result<()> {
        let file = File::open(&filepath).expect("Failed to open file: ");
        let reader = BufReader::new(file);
        let mut buf = BufWriter::new(Vec::with_capacity(100));

        let mut line_num = 1;
        let mut matched = false;

        buf.write_fmt(format_args!(
            "In file \x1b[0;34;1m{}\x1b[0m\n",
            filepath.as_path().to_str().unwrap()
        ))?;
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => return Ok(()),
            };
            if let Some(m) = pattern.find(&line) {
                matched = true;
                buf.write_fmt(format_args!("\x1b[0;33;1mline {}\x1b[0m: ", line_num))?;
                let bytes = line.as_bytes();
                let start = m.start();
                let end = m.end();
                let len = bytes.len();
                if start > 50 {
                    buf.write_fmt(format_args!(
                        "\x1b[0;36;1m...\x1b[0m{}",
                        String::from_utf8_lossy(&bytes[start - 50..start])
                    ))?;
                } else {
                    buf.write_all(&bytes[..start])?;
                }
                buf.write_all(b"\x1b[0;32;1m")?;
                buf.write_all(&bytes[start..end])?;
                buf.write_all(b"\x1b[0m")?;
                if len - end > 50 {
                    buf.write_fmt(format_args!(
                        "{}\x1b[0;36;1m...\x1b[0m",
                        String::from_utf8_lossy(&bytes[end..end + 50])
                    ))?;
                } else {
                    buf.write_all(&bytes[end..])?;
                }
                buf.write_all(b"\n")?;
            }
            line_num += 1;
        }
        buf.write_all(b"\n")?;

        let buf = buf.into_inner()?;

        if matched {
            sender
                .send(buf)
                .expect("Failed to send buffer to main thread: ");
        }

        Ok(())
    }

    fn match_content(&self, filepath: &PathBuf, writer: &mut BufWriter<StdoutLock>) -> Result<()> {
        let file = File::open(&filepath)?;
        let reader = BufReader::new(file);
        let mut buf = BufWriter::new(Vec::with_capacity(100));

        let mut line_num = 1;
        let mut matched = false;

        buf.write_fmt(format_args!(
            "In file \x1b[0;34;1m{}\x1b[0m\n",
            filepath.as_path().to_str().unwrap()
        ))?;
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => return Ok(()),
            };
            if let Some(m) = self.pattern.find(&line) {
                matched = true;
                buf.write_fmt(format_args!("\x1b[0;33;1mline {}\x1b[0m: ", line_num))?;

                let bytes = line.as_bytes();
                let start = m.start();
                let end = m.end();
                let len = bytes.len();
                if start > 50 {
                    buf.write_fmt(format_args!(
                        "\x1b[0;36;1m...\x1b[0m{}",
                        String::from_utf8_lossy(&bytes[start - 50..start])
                    ))?;
                } else {
                    buf.write_all(&bytes[..start])?;
                }
                buf.write_all(b"\x1b[0;32;1m")?;
                buf.write_all(&bytes[start..end])?;
                buf.write_all(b"\x1b[0m")?;
                if len - end > 50 {
                    buf.write_fmt(format_args!(
                        "{}\x1b[0;36;1m...\x1b[0m",
                        String::from_utf8_lossy(&bytes[end..end + 50])
                    ))?;
                } else {
                    buf.write_all(&bytes[end..])?;
                }
                buf.write_all(b"\n")?;
            }
            line_num += 1;
        }
        buf.write_all(b"\n")?;

        let buf = buf.into_inner()?;

        if matched {
            writer.write_all(&buf)?;
        }

        Ok(())
    }
}
