use std::{
    thread::sleep,
    time::Duration,
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
use constants::{WIFI_SSID, WIFI_PASSWORD};

fn heapless_str<const N: usize>(s: &str) -> heapless::String<N> {
    let mut out = heapless::String::<N>::new();
    out.push_str(s).expect("String too long");
    out
}

fn main(){
    esp_idf_sys::link_patches(); //Needed for esp32-rs
    esp_idf_svc::log::EspLogger::initialize_default();

    println!("Entered Main function!");
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

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
    while !wifi_driver.is_connected().unwrap(){
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
        sleep(Duration::from_secs(1));
    }
    println!("Should be connected now");
    loop{
        println!("IP info: {:?}", wifi_driver.sta_netif().get_ip_info().unwrap());
        sleep(Duration::new(10,0));
    }

}