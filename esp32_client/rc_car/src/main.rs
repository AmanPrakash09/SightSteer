use std::{
    thread::sleep,
    time::Duration,
    io::{BufRead, BufReader},
    net::{TcpStream, UdpSocket},
    str,
};
use esp_idf_sys as _;
use esp_idf_hal::{
    peripherals::Peripherals,
};
use esp_idf_svc::{
    wifi::EspWifi,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
};
use embedded_svc::wifi::{ClientConfiguration, Configuration};

mod constants;
use constants::{WIFI_SSID, WIFI_PASSWORD, DISCOVERY_PORT};

fn heapless_str<const N: usize>(s: &str) -> heapless::String<N> {
    let mut out = heapless::String::<N>::new();
    out.push_str(s).expect("String too long");
    out
}

fn main() -> std::io::Result<()>{
    esp_idf_sys::link_patches(); //Needed for esp32-rs
    esp_idf_svc::log::EspLogger::initialize_default();

    println!("Entered Main function!");
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    // Connect to WiFi first
    let mut wifi_driver = EspWifi::new(
        peripherals.modem,
        sys_loop,
        Some(nvs)
    ).unwrap();

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: heapless_str::<32>(WIFI_SSID),
        password: heapless_str::<64>(WIFI_PASSWORD),
        ..Default::default()
    })).unwrap();

    wifi_driver.start().unwrap();

    println!("Starting Wi-Fi scan...");
    match wifi_driver.scan() {
        Ok(networks) => {
            println!("Scan successful! Found {} networks.", networks.len());
            for net in networks {
                println!("Network: {:?}", net.ssid);
            }
        },
        Err(e) => {
            println!("Wi-Fi scan failed: {:?}", e);
        }
    }

    wifi_driver.connect().unwrap();

    // If wifi_driver.sta_netif().get_ip_info().unwrap() return 0.0.0.0, then any socket connection will fail
    // Need to wait until DHCP assign valid IP
    loop {
        let connected = wifi_driver.is_connected().unwrap();
        let ip_info = wifi_driver.sta_netif().get_ip_info().unwrap();
        if connected && ip_info.ip.to_string() != "0.0.0.0" {
            println!("Connected to IP Address: {}", ip_info.ip);
            break;
        }
        println!("Waiting for connection and until DHCP assigns vaild IP. Current IP: {}", ip_info.ip);
        sleep(Duration::from_secs(1));
    }

    // ----- UDP Discovery -----

    // Bind to discovery port to receive broadcasts
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT))?;
    let mut buf = [0; 128];

    // Wait for broadcast and extract server's IP and port
    let (server_ip, server_port) = loop {
        let (len, src) = socket.recv_from(&mut buf)?;
        let msg = str::from_utf8(&buf[..len]).unwrap_or("");

        if msg.starts_with("ECHO_SERVER:") {
            let parts: Vec<&str> = msg.split(':').collect();
            if parts.len() == 2 {
                let discovered_port_str = parts[1];
                if let Ok(discovered_port) = discovered_port_str.parse::<u16>() {
                    println!(
                        "Discovered server at {} on port {}",
                        src.ip(),
                        discovered_port
                    );
                    break (src.ip().to_string(), discovered_port);
                } else {
                    eprintln!("Failed to parse port from discovery message: {}", msg);
                }
            } else {
                eprintln!("Invalid discovery message format: {}", msg);
            }
        }
    };

    // ----- TCP Connection -----

    // Connect to server via TCP using extractd server IP and port
    let addr = format!("{}:{}", server_ip, server_port);
    println!("Connecting to server at {}...", addr);

    // If ESP32 receives UDP broadcast before server is ready to accept TCP connections, then there is a race condition between discovery and server readiness
    // Need to loop until server is ready and connection is made
    // Also, if ESP32 loses WiFi connection during TCP, try to reconnect to WiFi
    loop {
        if !wifi_driver.is_connected().unwrap_or(false) {
            println!("WiFi disconnected. Neet to reconnect ...");
            let _ = wifi_driver.connect();
            sleep(Duration::from_secs(1));
            continue;
        }

        match TcpStream::connect(&addr) {
            Ok(stream) => {
                println!("Connected!");
                // Read and print data from server
                let reader = BufReader::new(stream);
                for line in reader.lines() {
                    match line {
                        Ok(data) => println!("Received: {}", data),
                        Err(e) => {
                            println!("Connection error: {:?}. Reconnecting ...", e);
                            break; // If TCP error, break from reading lines and try reconnecting
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("TCP connection failed: {:?}, retrying ...", e);
                sleep(Duration::from_secs(1));
            }
        }
    };
}