//! Test InitGame + Map with player creature on it
//!
//! The client accepts InitGame and waits, but doesn't render.
//! Maybe it needs the player creature to be IN the map data.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::thread;

fn main() {
    println!("=== Tibia 1.03 Map + Creature Test ===\n");
    println!("Listening on 127.0.0.1:7171\n");

    let listener = TcpListener::bind("127.0.0.1:7171").expect("Failed to bind");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("[CONNECT] {:?}", stream.peer_addr());
                let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));

                if !read_login(&mut stream) { continue; }

                send_init_with_creature(&mut stream);

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

    if body.len() >= 37 {
        let name_bytes = &body[7..37];
        let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(30);
        let name = String::from_utf8_lossy(&name_bytes[..name_end]);
        println!("[LOGIN] Player: {}\n", name);
    }
    true
}

fn send_init_with_creature(stream: &mut TcpStream) {
    let player_id: u32 = 0x10000001;
    let player_name = "TestPlayerName";

    // Map center position
    let center_x: u16 = 100;
    let center_y: u16 = 100;
    let center_z: u8 = 7;

    println!("[SENDING INIT SEQUENCE]\n");

    // 1. InitGame - minimal version that worked
    let mut init = Vec::new();
    init.push(0x0A);
    init.extend(&player_id.to_le_bytes());
    send(stream, &init, "InitGame (0x0A)");

    // 2. FullMap with player creature on center tile
    let mut map = Vec::new();
    map.push(0x64);  // opcode
    map.extend(&center_x.to_le_bytes());
    map.extend(&center_y.to_le_bytes());
    map.push(center_z);

    // Map is 18x14 = 252 tiles
    // Player should be at center (tile index 126 = 9*14 + 0... actually let's calculate)
    // Center tile is at (9, 7) in the 18x14 grid = index 7*18 + 9 = 135
    let player_tile_index = 7 * 18 + 9;  // row 7, column 9

    for i in 0..(18 * 14) {
        // Ground tile - try different IDs
        // ID 106 is often grass in old Tibia
        let ground_id: u16 = 106;
        map.extend(&ground_id.to_le_bytes());

        if i == player_tile_index {
            // Add player creature on this tile
            // Creature format: 0x0061 or 0x0062 = new creature marker
            map.extend(&0x0061u16.to_le_bytes());  // known creature marker
            map.extend(&player_id.to_le_bytes());  // creature ID

            // OR use 0x0062 for NEW creature with full data:
            // Let's try the simpler "known" marker first
        }

        // Tile terminator
        map.push(0xFF);
        map.push(0xFF);
    }

    send(stream, &map, "FullMap with player (0x64)");

    println!("\n[NOTE] Player creature added at tile index {}", player_tile_index);
    println!("[NOTE] Using ground item ID 106 (common grass)");
    println!("");
}

fn send(stream: &mut TcpStream, body: &[u8], name: &str) {
    let len = body.len() as u16;
    let mut pkt = len.to_le_bytes().to_vec();
    pkt.extend(body);

    print!("  {}: ", name);
    if stream.write_all(&pkt).is_ok() {
        let _ = stream.flush();
        if body.len() <= 40 {
            println!("{:02X?}", body);
        } else {
            println!("{:02X?}... ({} bytes total)", &body[..30], body.len());
        }
    } else {
        println!("WRITE ERROR");
    }
    thread::sleep(Duration::from_millis(50));
}

fn monitor_client(stream: &mut TcpStream) {
    println!("[MONITORING] Waiting for client (30s timeout)...");
    println!("[TIP] Try pressing arrow keys in the client!\n");

    loop {
        let mut len_buf = [0u8; 2];
        match stream.read_exact(&mut len_buf) {
            Ok(_) => {
                let length = u16::from_le_bytes(len_buf) as usize;
                if length > 0 && length < 10000 {
                    let mut body = vec![0u8; length];
                    if stream.read_exact(&mut body).is_ok() {
                        let opcode = body.first().copied().unwrap_or(0);
                        println!("┌─────────────────────────────────────────────");
                        println!("│ [RECV] opcode=0x{:02X} ({} bytes)", opcode, length);

                        match opcode {
                            0x65 => println!("│        → WALK NORTH! Client is responding!"),
                            0x66 => println!("│        → WALK EAST!"),
                            0x67 => println!("│        → WALK SOUTH!"),
                            0x68 => println!("│        → WALK WEST!"),
                            0x6F..=0x72 => println!("│        → TURN command"),
                            0x96 => println!("│        → SAY/CHAT"),
                            _ => println!("│        → Unknown opcode"),
                        }
                        println!("│        Body: {:02X?}", &body[..body.len().min(20)]);
                        println!("└─────────────────────────────────────────────");
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(_) => {
                println!("[DISCONNECTED]");
                break;
            }
        }
    }
}