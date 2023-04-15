use std::time::Instant;
use std::{env, time::Duration};
use std::error::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpSocket, UdpSocket, TcpStream};

type Socket = UdpSocket;

fn print_timings(mut timings: Vec<Duration>) {
    timings.sort();
    println!("min    = {}us", timings[0].as_micros());
    println!("max    = {}us", timings[timings.len() - 1].as_micros());
    println!("avg    = {}us", timings.iter().map(|x| x.as_micros()).sum::<u128>() / timings.len() as u128);
    for quantile in [50., 90., 95., 99., 99.9, 99.99] {
        let idx = (timings.len() as f64 * (quantile / 100.)) as usize;
        println!("p{:<5} = {}us", quantile, timings[idx].as_micros());
    }
}
const MAX_DATAGRAM_SIZE: usize = 65_507;

async fn run_udp(local: SocketAddr, remote: SocketAddr, n_tries: usize) -> Result<Vec<Duration>, Box<dyn Error>> {
    let socket = Socket::bind(local).await?;
    socket.connect(&remote).await?;

    let data: Vec<u8> = "proto!".to_owned().into_bytes();
    let expected: Vec<u8> = "proto!proto!".to_owned().into_bytes();

    let mut timings: Vec<Duration> = vec![];
    timings.reserve(n_tries);


    for _ in 0..n_tries {
        let mut buffer = vec![0u8; MAX_DATAGRAM_SIZE];
        let start = Instant::now();
        socket.send(&data).await?;
        let len = socket.recv(&mut buffer).await?;
        timings.push(start.elapsed());

        assert_eq!(buffer[..len], expected);
        // println!(
        //     "Received {} bytes:\n{}",
        //     len,
        //     String::from_utf8_lossy(&data[..len])
        // );

    }

    Ok(timings)
}

async fn run_tcp(_local: SocketAddr, remote: SocketAddr, n_tries: usize) -> Result<Vec<Duration>, Box<dyn Error>> {
    let mut stream = TcpStream::connect(remote).await?;


    let data: Vec<u8> = "proto!".to_owned().into_bytes();
    let expected: Vec<u8> = "proto!proto!".to_owned().into_bytes();

    let mut timings: Vec<Duration> = vec![];
    timings.reserve(n_tries);


    for _ in 0..n_tries {
        let mut buffer = vec![0u8; 12];
        let start = Instant::now();
        stream.write_all(&data).await?;
        let len = stream.read_exact(&mut buffer).await?;
        timings.push(start.elapsed());

        assert_eq!(buffer[..len], expected);
        // println!(
        //     "Received {} bytes:\n{}",
        //     len,
        //     String::from_utf8_lossy(&data[..len])
        // );

    }

    Ok(timings)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let remote_addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "[2a02:6b8:c0b:3e19:0:519:3b8e:0]:4080".into())
        .parse()?;

    let n_tries: usize = env::args().nth(2).unwrap().parse()?;
    let udp = env::args().nth(3).unwrap_or_else(|| "tcp".to_string()) == "udp";

    // We use port 0 to let the operating system allocate an available port for us.
    let local_addr: SocketAddr = "[::]:0".parse()?;

    if udp {
        print_timings(run_udp(local_addr, remote_addr, n_tries).await?);
    } else {
        print_timings(run_tcp(local_addr, remote_addr, n_tries).await?);
    }


    Ok(())
}
