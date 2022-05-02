use std::io::{self, Read};
use std::net::{IpAddr, SocketAddr};
use std::process::id;
use std::time::{Duration, Instant};

use clap::{App, Arg};
use crossterm::style::Stylize;
use rand::random;
use socket2::{Domain, Protocol, Socket, Type};
use thiserror::Error;

use crate::icmp::{self, EchoReply, EchoRequest, IcmpV4, IcmpV6, HEADER_SIZE};
use crate::ip::{self, *};

#[derive(Debug)]
struct Data {
    ttl: Option<u8>,
    time: Duration,
}

struct Statistics {
    total_packet_cnt: u32,
    lost_packet_cnt: u32,
    total_time: Duration,
    max_time: Duration,
    min_time: Duration,
}

impl Statistics {
    fn new() -> Statistics {
        Statistics {
            total_packet_cnt: 0,
            lost_packet_cnt: 0,
            total_time: Duration::new(0, 0),
            max_time: Duration::new(0, 0),
            min_time: Duration::new(u64::MAX >> 1, u32::MAX),
        }
    }
}

#[derive(Debug, Error)]
pub enum PingError {
    #[error("invaild ip packet: {:?}", .0)]
    InvalidIpPacket(#[from] ip::Error),
    #[error("invaild icmp packet: {:?}", .0)]
    InvaildICMPPacket(#[from] icmp::DecodeError),
    #[error("io error: {error}")]
    IoError {
        #[from]
        #[source]
        error: std::io::Error,
    },
}
pub type PingResult<T> = Result<T, PingError>;

pub struct PingApp {
    host: Option<String>,
    addr: IpAddr,
    timeout: Option<Duration>,
    ttl: Option<u32>,
    ident: Option<u16>,
    seq_cnt: Option<u16>,
    size: Option<usize>,
    cnt: u32,
}

impl PingApp {
    pub fn from_args() -> PingApp {
        let matches = App::new("ping")
            .arg(
                Arg::new("REMOTE")
                    .takes_value(true)
                    .help("Remote ip address or url"),
            )
            .arg(
                Arg::new("TIMEOUT")
                    .takes_value(true)
                    .short('t')
                    .long("time-out")
                    .help("Set timeout"),
            )
            .arg(
                Arg::new("SIZE")
                    .takes_value(true)
                    .short('n')
                    .long("size")
                    .help("Set the ping data size (bytes)"),
            )
            .arg(
                Arg::new("TTL")
                    .takes_value(true)
                    .short('l')
                    .long("ttl")
                    .help("Set the ttl value"),
            )
            .arg(
                Arg::new("ID")
                    .takes_value(true)
                    .short('i')
                    .long("id")
                    .help("Set the identifier field in ICMP header"),
            )
            .arg(
                Arg::new("SEQ")
                    .takes_value(true)
                    .short('s')
                    .long("seq")
                    .help("Set the sequence num in ICMP header"),
            )
            .arg(
                Arg::new("CNT")
                    .takes_value(true)
                    .short('c')
                    .long("cnt")
                    .help("Set ping data packet count"),
            )
            .about("Ping a remote ip (ipv4 or ipv6).")
            .author("朕与将军解战袍, 1393323447@qq.com")
            .version("0.1.0")
            .get_matches();

        let host = matches
            .value_of("REMOTE")
            .expect("Please persent a ip addresss or an url");
        let (host, addr) = match host.parse::<IpAddr>() {
            Ok(ip) => (None, ip),
            Err(_) => {
                let (host_name, ip) = look_up_ip(host).unwrap();
                (Some(host_name), ip)
            }
        };
        let timeout = matches.value_of("TIMEOUT").map(parse_timeout);
        let ttl = matches.value_of("TTL").map(|ttl| ttl.parse().unwrap());
        let ident = matches.value_of("ID").map(|id| id.parse().unwrap());
        let seq_cnt = matches.value_of("SEQ").map(|seq| seq.parse().unwrap());
        let size = matches.value_of("SIZE").map(|size| size.parse().unwrap());
        let cnt = matches
            .value_of("CNT")
            .map(|cnt| cnt.parse().unwrap())
            .unwrap_or(4);

        PingApp {
            host,
            addr,
            timeout,
            ttl,
            ident,
            seq_cnt,
            size,
            cnt,
        }
    }

    pub fn run(&self) {
        let ip = format!("{}", self.addr).blue();
        let size = format!("{}", self.size.unwrap_or(32)).blue();

        match self.host {
            Some(ref host) => {
                let host = format!("{}", host).green();
                println!("ping {} [{}] with {} bytes of data: ", host, ip, size);
            }
            None => println!("ping {} with {} bytes of data: ", ip, size),
        }

        let mut stats = Statistics::new();
        stats.total_packet_cnt = self.cnt;

        for _ in 0..self.cnt {
            match self.ping() {
                Ok(data) => {
                    let ttl = data.ttl.map(|ttl| format!("{}", ttl).yellow());
                    let time = format!("{:?}", data.time).green();
                    match ttl {
                        Some(ttl) => println!(
                            "Reply from {}: bytes={} time={} ttl={}",
                            ip, size, time, ttl
                        ),
                        None => println!("Reply from {}: bytes={} time ={}", ip, size, time),
                    }
                    stats.total_time += data.time;
                    stats.max_time = stats.max_time.max(data.time);
                    stats.min_time = stats.min_time.min(data.time);
                }
                Err(err) => {
                    stats.lost_packet_cnt += 1;
                    stats.max_time = Duration::from_millis(9999);
                    let err_msg = format!("{:?}", &err).red();
                    println!("Ping error: {}", err_msg);
                }
            }
        }

        if stats.min_time > stats.max_time {
            // 超时
            stats.min_time = stats.max_time;
            stats.total_time = stats.max_time * self.cnt;
        }

        let total = format!("{}", stats.total_packet_cnt).blue();
        let recv = format!("{}", stats.total_packet_cnt - stats.lost_packet_cnt).green();
        let lost = format!("{}", stats.lost_packet_cnt).red();
        let lost_presentage = {
            let presentage = stats.lost_packet_cnt as f64 / stats.total_packet_cnt as f64 * 100.0;
            let presentage_str = format!("{}", presentage);

            if presentage > 40.0 {
                presentage_str.red()
            } else if presentage > 20.0 {
                presentage_str.yellow()
            } else {
                presentage_str.green()
            }
        };
        let max_time = format!("{:#2?}", stats.max_time).green();
        let min_time = format!("{:#2?}", stats.min_time).green();
        let avg_time = format!("{:#2?}", stats.total_time / stats.total_packet_cnt).green();
        println!("Ping statistics for {}: ", ip);
        println!(
            "    Packets: Sent = {}, Received = {}, Loss = {} ({}% loss)",
            total, recv, lost, lost_presentage
        );
        println!("Approximate round trip times in milli-seconds: ");
        println!(
            "    Minimum = {}, Maximum = {}, Average = {}",
            min_time, max_time, avg_time
        );
    }

    fn ping(&self) -> PingResult<Data> {
        let timeout = match self.timeout {
            Some(timeout) => Some(timeout),
            None => Some(Duration::from_secs(4)),
        };

        let dest = SocketAddr::new(self.addr, 0);

        let size = self.size.unwrap_or(32);
        let cap = size + HEADER_SIZE;
        let mut buffer = vec![0u8; cap];

        let mut payload = Vec::with_capacity(size);
        for _ in 0..size {
            payload.push(random());
        }

        let request = EchoRequest {
            ident: self.ident.unwrap_or(id() as u16),
            seq_cnt: self.seq_cnt.unwrap_or(1),
            payload: &payload,
        };

        let mut socket = if dest.is_ipv4() {
            request.encode::<IcmpV4>(&mut buffer);
            Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?
        } else {
            request.encode::<IcmpV6>(&mut buffer);
            Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6))?
        };

        socket.set_ttl(self.ttl.unwrap_or(64))?;

        socket.set_write_timeout(timeout)?;

        socket.send_to(&buffer, &dest.into())?;

        socket.set_read_timeout(timeout)?;

        let timer = Instant::now();
        let mut buffer = [0u8; 2048];
        let size = socket.read(&mut buffer)?;

        let mut data = Data {
            ttl: None,
            time: timer.elapsed(),
        };

        let _reply = if dest.is_ipv4() {
            let ipv4_packet = IpV4Packet::decode(&buffer[..size])
                .map_err(|err| PingError::InvalidIpPacket(err))?;
            data.ttl = Some(ipv4_packet.ttl);

            EchoReply::decode::<IcmpV4>(ipv4_packet.data)
                .map_err(|err| PingError::InvaildICMPPacket(err))?
        } else {
            EchoReply::decode::<IcmpV6>(&buffer[..size])
                .map_err(|err| PingError::InvaildICMPPacket(err))?
        };

        Ok(data)
    }
}

fn parse_timeout(timeout: &str) -> Duration {
    let timeout = timeout.trim();
    let mut num = String::new();
    let mut unit = String::new();

    for ch in timeout.chars() {
        if ch.is_digit(10) || ch == '.' {
            num.push(ch);
        } else if ch.is_alphabetic() {
            unit.push(ch);
        }
    }

    let num = num
        .parse()
        .unwrap_or_else(|_| panic!("Invaild timeout num: {}", num));
    let unit_str: &str = &unit;
    match unit_str {
        "" | "ms" => Duration::from_millis(num),
        "us" | "µs" => Duration::from_micros(num),
        "ns" => Duration::from_nanos(num),
        _ => panic!("Invaild timeout unit: {}", unit),
    }
}

fn look_up_ip(host: &str) -> io::Result<(String, IpAddr)> {
    let resolver = trust_dns_resolver::Resolver::default()?;
    let lookup = resolver.lookup_ip(host)?;

    let record = lookup
        .as_lookup()
        .record_iter()
        .find(|record| record.data().is_some() && record.data().unwrap().to_ip_addr().is_some())
        .unwrap_or_else(|| panic!("Cannot resolve ip for {}", host));
    let host_name = record.name().to_string();
    let ip = record.data().unwrap().to_ip_addr().unwrap();
    Ok((host_name, ip))
}
