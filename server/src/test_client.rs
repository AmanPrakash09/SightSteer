// ==============================
// CLIENT: test_client.rs
// ==============================
//
// - Listens for a UDP broadcast from the server
// - Extracts the server's IP
// - Connects to the server via TCP on a fixed ECHO_PORT
// - Prints received JSON messages

use std::io::{BufRead, BufReader};
use std::net::{TcpStream, UdpSocket};
use std::str;

mod constants;
use constants::{DISCOVERY_PORT, ECHO_PORT};

fn main() -> std::io::Result<()> {
    println!("Client: Listening for UDP broadcast on port {}...", DISCOVERY_PORT);

    // Bind to discovery port to receive broadcasts
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT))?;
    let mut buf = [0; 128];

    // Wait for broadcast and extract server's IP
    let server_ip = loop {
        let (len, src) = socket.recv_from(&mut buf)?;
        let msg = str::from_utf8(&buf[..len]).unwrap_or("");

        if msg.starts_with("ECHO_SERVER:") {
            println!("Discovered server at {}", src.ip());
            break src.ip().to_string();
        }
    };

    // Connect to server via TCP using fixed ECHO_PORT
    let addr = format!("{}:{}", server_ip, ECHO_PORT);
    println!("Connecting to server at {}...", addr);
    let stream = TcpStream::connect(addr)?;
    println!("Connected!");

    // Read and print data from server
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line?;
        println!("Received: {}", line);
    }

    Ok(())
}