// ==============================
// SERVER: main.rs
// ==============================
//
// - Broadcast its IP and port over UDP so clients can discover it
// - Accept incoming TCP connections from clients (e.g., ESP32 or test client)
// - Run a Python script that outputs {"state": ..., "angle": ...} as JSON
// - Stream this JSON over TCP to the connected client
//
// IPs and Ports:
// - Discovery IP  : 255.255.255.255 (UDP broadcast to local network)
// - DISCOVERY_PORT: UDP port that clients listen on to discover the server
// - ECHO_PORT     : TCP port where clients connect to receive data
//
// Localhost (127.0.0.1):
// - Used when running and testing the server + client on the same machine.
//
// Note: The server listens on 0.0.0.0 to accept connections from any local interface.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

mod constants;
use constants::{DISCOVERY_PORT, ECHO_PORT};

fn broadcast() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    let msg = format!("ECHO_SERVER:{}", ECHO_PORT);

    loop {
        socket.send_to(msg.as_bytes(), format!("255.255.255.255:{}", DISCOVERY_PORT))?;
        thread::sleep(Duration::from_secs(2));
    }
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    println!("Client connected!");

    // Launch Python using venv interpreter (adjust path for your OS)
    let mut child = Command::new("../remote_control/venv/Scripts/python") // Windows
        // "../remote_control/venv/bin/python3" for macOS/Linux
        .arg("../remote_control/hand_recognition.py")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run hand_recognition.py");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line?;
        println!("JSON Output: {}", line);

        // Send JSON to client
        stream.write_all(line.as_bytes())?;
        stream.write_all(b"\n")?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    thread::spawn(broadcast);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", ECHO_PORT))?;
    println!("Server listening on port {}", ECHO_PORT);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream)?;
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }

    Ok(())
}