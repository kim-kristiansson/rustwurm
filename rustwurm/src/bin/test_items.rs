//! Test different ground item IDs
//!
//! We don't know what item IDs exist in Tibia 1.03's tibia.dat
//! This lets you try different IDs to find valid ones.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::thread;

fn main() {
    let item_id: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    println!("=== Tibia 1.03 Item ID Tester ===");
    println!("Testing ground item ID: {}", item_id);
    println!("Usage: cargo run --bin test_items <item_id>\n");
    println!("Listening on 127.0.0.1:7171\n");

    let listener = TcpListener::bind("127.0.0.1:7171").expect("Failed to bind");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("[CONNECT] {:?}", stream.peer_addr());
                let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));

                if !read_login(&mut stream) { continue; }

                send_init_with_item_id(&mut stream, item_id);

                monitor_client(&mut stream);
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }
}

fn read_login(stream: &mut TcpStream) -> bool {
    let mut len_buf = [0u8; 2];
    if stream.read_exact(&mut len_buf).is_err() { return false; }
    let length = u16::from_le_bytes(len_buf) as usize;
    let mut body = vec![0u8; length];
    if stream.read_exact(&mut body).is_err() { return false; }
    println!("[LOGIN] OK\n");
    true
}

fn send_init_with_item_id(stream: &mut TcpStream, ground_id: u16) {
    let player_id: u32 = 0x10000001;
    let center_x: u16 = 100;
    let center_y: u16 = 100;
    let center_z: u8 = 7;

    // 1. InitGame
    let mut init = Vec::new();
    init.push(0x0A);
    init.extend(&player_id.to_le_bytes());
    send(stream, &init, "InitGame");

    // 2. FullMap
    let mut map = Vec::new();
    map.push(0x64);
    map.extend(&center_x.to_le_bytes());
    map.extend(&center_y.to_le_bytes());
    map.push(center_z);

    // 252 tiles
    let player_tile = 7 * 18 + 9;  // center-ish

    for i in 0..(18 * 14) {
        map.extend(&ground_id.to_le_bytes());

        // Add player creature on center tile
        if i == player_tile {
            // Try 0x0062 = new creature marker
            map.extend(&0x0062u16.to_le_bytes());
            map.extend(&0u32.to_le_bytes());     // remove ID (0 = new)
            map.extend(&player_id.to_le_bytes()); // creature ID
            // Name
            let name = "Player";
            map.extend(&(name.len() as u16).to_le_bytes());
            map.extend(name.as_bytes());
            map.push(100);  // health %
            map.push(2);    // direction (south)
            // Outfit
            map.extend(&128u16.to_le_bytes());  // lookType
            map.push(0);    // head color
            map.push(0);    // body color
            map.push(0);    // legs color
            map.push(0);    // feet color
            // Light
            map.push(0);    // light intensity
            map.push(0);    // light color
            // Speed
            map.extend(&220u16.to_le_bytes());
        }

        map.push(0xFF);
        map.push(0xFF);
    }

    send(stream, &map, &format!("FullMap (ground={})", ground_id));
    println!("");
}

fn send(stream: &mut TcpStream, body: &[u8], name: &str) {
    let len = body.len() as u16;
    let mut pkt = len.to_le_bytes().to_vec();
    pkt.extend(body);
    print!("  {}: {} bytes ", name, body.len());
    if stream.write_all(&pkt).is_ok() {
        let _ = stream.flush();
        println!("OK");
    } else {
        println!("ERROR");
    }
    thread::sleep(Duration::from_millis(50));
}

fn monitor_client(stream: &mut TcpStream) {
    println!("[WAITING] Try pressing arrow keys in client...\n");

    loop {
        let mut len_buf = [0u8; 2];
        match stream.read_exact(&mut len_buf) {
            Ok(_) => {
                let length = u16::from_le_bytes(len_buf) as usize;
                if length > 0 && length < 10000 {
                    let mut body = vec![0u8; length];
                    if stream.read_exact(&mut body).is_ok() {
                        let op = body.first().copied().unwrap_or(0);
                        print!("[RECV] 0x{:02X} ", op);
                        match op {
                            0x65 => println!("WALK NORTH - IT WORKS!"),
                            0x66 => println!("WALK EAST - IT WORKS!"),
                            0x67 => println!("WALK SOUTH - IT WORKS!"),
                            0x68 => println!("WALK WEST - IT WORKS!"),
                            _ => println!("(len={})", length),
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock ||
                e.kind() == std::io::ErrorKind::TimedOut => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(_) => { println!("[DISCONNECTED]"); break; }
        }
    }
}