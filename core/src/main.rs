use std::net::SocketAddr;

use custom_ddp_controller::{
    pixels::{pixelstrip::PixelStrip, pixelstripmanager::PixelStripManager},
    services::LedCommandHandler,
    DISPLAY_FREQUENCY, LED_COUNT,
};
use ddp_rs::{connection::DDPConnection, protocol};

use std::{
    env,
    net::{IpAddr, Ipv4Addr},
};

use hyper_services::{service::stateful_service::StatefulService};

#[tokio::main]
async fn main() {
    println!("Starting.");

    let args: Vec<String> = env::args().collect();

    let device_ip: Ipv4Addr = match args.get(1) {
        Some(ip) => match ip.parse::<Ipv4Addr>() {
            Ok(ip) => ip,
            Err(e) => {
                eprintln!("Couldn't parse provided IP address {}. {:?}", ip, e);
                return;
            }
        },
        None => {
            eprintln!("Provide the device IP address as the first argument.");
            return;
        }
    };

    let device_port = match args.get(2) {
        Some(port) => match port.parse::<u16>() {
            Ok(port) => port,
            Err(e) => {
                eprintln!("Invalid port {}", port);
                eprintln!("{:?}", e);
                return;
            }
        },
        None => {
            eprintln!("Provide the device port as the second argument.");
            return;
        }
    };

    let service_port = match args.get(3) {
        Some(port) => match port.parse::<u16>() {
            Ok(port) => port,
            Err(e) => {
                eprintln!("Invalid port {}", port);
                eprintln!("{:?}", e);
                return;
            }
        },
        None => {
            eprintln!("Provide the service port as the third argument.");
            return;
        }
    };

    let pixel_strip = PixelStrip::create(LED_COUNT);

    println!("Binding outbound port.");
    let outbound_port = std::net::UdpSocket::bind("0.0.0.0:4048"); // can be any unused port on 0.0.0.0, but protocol recommends 4048

    let outbound_port = match outbound_port {
        Ok(port) => port,
        Err(e) => {
            eprintln!("Couldn't bind port.");
            eprintln!("{:?}", e);
            return;
        }
    };

    let socket_address: SocketAddr = SocketAddr::new(IpAddr::V4(device_ip), device_port);

    println!("Creating DDP connection at {:?}.", socket_address);
    let conn = DDPConnection::try_new(
        socket_address,                   // The IP address of the device followed by :4048
        protocol::PixelConfig::default(), // Default is RGB, 8 bits ber channel
        protocol::ID::Default,
        outbound_port,
    );

    let conn = match conn {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Couldn't create DDP Connection.");
            eprintln!("{:?}", e);
            return;
        }
    };

    //Scope the pixel strip manager creation to avoid deadlock.
    let event_server = {
        println!("Creating pixel strip manager.");
        let pixel_strip_manager = PixelStripManager::new(pixel_strip, DISPLAY_FREQUENCY, conn);

        //demos::red_green_blue(conn, pixels)?;
        //demos::hue_progression(conn, pixels)?;
        //demos::rainbow_oscillation(conn, pixel_strip).unwrap();

        println!("Creating LED Command Handler.");

        let handler = LedCommandHandler::new(pixel_strip_manager);

        println!("Starting DDP Service");
        let service = StatefulService::create(handler);
        service.start(
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            service_port,
            hyper_services::ConnectionProperties::default()
        )
    };

    println!("DDP Service Running");

    match event_server.await {
        Ok(_) => println!("Closed DDP Service Gracefully"),
        Err(e) => {
            println!("DDP Service Failure");
            println!("{}", e.to_string());
        }
    };
}
