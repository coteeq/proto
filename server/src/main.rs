use std::error::Error;
use std::net::SocketAddr;
use std::{env, io};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpListener, UdpSocket, TcpStream};

type Socket = UdpSocket;

struct Server {
    socket: Socket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
}
struct TcpServer {
    socket: TcpListener,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
}

fn transform(payload: Vec<u8>, size: usize) -> Vec<u8> {
    let mut first = payload[..size].to_vec();
    let mut second = first.clone();
    first.append(&mut second);
    first
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            socket,
            mut buf,
            mut to_send,
        } = self;

        loop {
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            if let Some((size, peer)) = to_send {
                let amt = socket.send_to(&buf[..size], &peer).await?;
                buf = vec![0; 6];

                // println!("Echoed {}/{} bytes to {}", amt, size, peer);
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            let (size_from_wire, addr) = socket.recv_from(&mut buf).await?;
            buf = transform(buf, size_from_wire);
            to_send = Some((buf.len(), addr));
        }
    }
}

async fn handle_conn(mut sock: TcpStream) -> Result<(), io::Error> {
    loop {
        let mut buf = vec![0; 6];
        let size_from_wire = sock
            .read_exact(&mut buf)
            .await?;
        // If we're here then `to_send` is `None`, so we take a look for the
        // next message we're going to echo back.
        buf = transform(buf, size_from_wire);
        sock
            .write_all(&buf)
            .await?
    }
}



impl TcpServer {
    async fn run(self) -> Result<(), io::Error> {
        loop {
            if let Err(e) = handle_conn(self.socket.accept().await?.0).await {
                println!("dropped: {}", e);
            }
        }
    }

}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| ":::4080".to_string());

    let udp = env::args().nth(2).unwrap_or_else(|| "tcp".to_string()) == "udp";

    if udp {
        let socket = Socket::bind(&addr).await?;
        println!("Listening on: {}", socket.local_addr()?);

        let server = Server {
            socket,
            buf: vec![0; 6],
            to_send: None,
        };

        // This starts the server task.
        server.run().await?;
    } else {
        let listener = TcpListener::bind(&addr).await?;
        println!("Listening on: {}", listener.local_addr()?);

        let server = TcpServer {
            socket: listener,
            buf: vec![0; 6],
            to_send: None,
        };

        // This starts the server task.
        server.run().await?;
    }

    Ok(())
}
