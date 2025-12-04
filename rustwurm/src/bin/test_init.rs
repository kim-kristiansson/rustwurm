//! Test different InitGame formats to find what Tibia 1.03 accepts
//!
//! The client disconnects immediately, so the issue is likely in InitGame.
//! This tests increasingly minimal versions.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::thread;

fn main() {
    println!("=== Tibia 1.03 InitGame Format Tester ===");
    println!("Testing different InitGame packet formats.\n");
    println!("Listening on 127.0.0.1:7171\n");

    let listener = TcpListener::bind("127.0.0.1:7171").expect("Failed to bind");

    // Which test variant to try (change this number to test different formats)
    let test_variant = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    println!("Using test variant: {}", test_variant);
    println!("Run with: cargo run --bin test_init <number>\n");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("[CONNECT] {:?}", stream.peer_addr());
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

                if !read_login(&mut stream) { continue; }

                test_init_variant(&mut stream, test_variant);

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

fn test_init_variant(stream: &mut TcpStream, variant: u32) {
    let player_id: u32 = 0x10000001;

    match variant {
        1 => {
            println!("=== VARIANT 1: Minimal InitGame (just player ID) ===");
            // Maybe 1.03 just wants: opcode + playerId
            let mut p = Vec::new();
            p.push(0x0A);
            p.extend(&player_id.to_le_bytes());
            send(stream, &p, "InitGame (minimal)");
        }

        2 => {
            println!("=== VARIANT 2: InitGame + sessionFlags only ===");
            let mut p = Vec::new();
            p.push(0x0A);
            p.extend(&player_id.to_le_bytes());
            p.extend(&0u16.to_le_bytes());  // sessionFlags
            send(stream, &p, "InitGame (+sessionFlags)");
        }

        3 => {
            println!("=== VARIANT 3: InitGame without welcomeText ===");
            let mut p = Vec::new();
            p.push(0x0A);
            p.extend(&player_id.to_le_bytes());
            p.extend(&0u16.to_le_bytes());  // sessionFlags
            p.push(0);                       // canReportBugs
            // NO welcomeText field
            send(stream, &p, "InitGame (no welcomeText)");
        }

        4 => {
            println!("=== VARIANT 4: Full InitGame (current) ===");
            let mut p = Vec::new();
            p.push(0x0A);
            p.extend(&player_id.to_le_bytes());
            p.extend(&0u16.to_le_bytes());  // sessionFlags
            p.push(0);                       // canReportBugs
            p.extend(&0u16.to_le_bytes());  // welcomeText len=0
            send(stream, &p, "InitGame (full)");
        }

        5 => {
            println!("=== VARIANT 5: Just TextMessage (skip InitGame) ===");
            // Maybe client expects a different first packet?
            let msg = "Welcome!";
            let mut p = Vec::new();
            p.push(0xB4);
            p.push(0x12);
            p.extend(&(msg.len() as u16).to_le_bytes());
            p.extend(msg.as_bytes());
            send(stream, &p, "TextMessage only");
        }

        6 => {
            println!("=== VARIANT 6: MOTD style (0x14) ===");
            // Maybe 1.03 expects MOTD format like login server?
            let msg = "Welcome to Tibia!";
            let mut p = Vec::new();
            p.push(0x14);  // MOTD opcode from login server
            p.extend(&(msg.len() as u16).to_le_bytes());
            p.extend(msg.as_bytes());
            send(stream, &p, "MOTD (0x14)");
        }

        7 => {
            println!("=== VARIANT 7: InitGame with actual welcome string ===");
            let welcome = "Welcome!";
            let mut p = Vec::new();
            p.push(0x0A);
            p.extend(&player_id.to_le_bytes());
            p.extend(&0u16.to_le_bytes());  // sessionFlags
            p.push(0);                       // canReportBugs
            p.extend(&(welcome.len() as u16).to_le_bytes());
            p.extend(welcome.as_bytes());
            send(stream, &p, "InitGame (with welcome)");
        }

        8 => {
            println!("=== VARIANT 8: Try opcode 0x01 instead of 0x0A ===");
            // Maybe game init uses different opcode in 1.03?
            let mut p = Vec::new();
            p.push(0x01);
            p.extend(&player_id.to_le_bytes());
            send(stream, &p, "Init with opcode 0x01");
        }

        9 => {
            println!("=== VARIANT 9: Character list response (login server style) ===");
            // Maybe the same socket expects login server responses?
            // MOTD (0x14) + CharList (0x64)
            let motd = "Welcome!";
            let mut p = Vec::new();
            p.push(0x14);  // MOTD
            p.extend(&(motd.len() as u16).to_le_bytes());
            p.extend(motd.as_bytes());
            send(stream, &p, "MOTD (0x14)");

            // Then character list
            let mut p = Vec::new();
            p.push(0x64);  // CharList
            p.push(1);     // 1 character
            let name = "TestPlayer";
            p.extend(&(name.len() as u16).to_le_bytes());
            p.extend(name.as_bytes());
            let world = "Rustwurm";
            p.extend(&(world.len() as u16).to_le_bytes());
            p.extend(world.as_bytes());
            p.extend(&[127u8, 0, 0, 1]);  // IP 127.0.0.1
            p.extend(&7172u16.to_le_bytes());  // port
            send(stream, &p, "CharList (0x64)");
        }

        10 => {
            println!("=== VARIANT 10: Empty response (see what client does) ===");
            // Send nothing, just wait
            println!("  [Sending nothing, waiting for client...]");
        }

        11 => {
            println!("=== VARIANT 11: InitGame + Map immediately ===");
            let x: u16 = 100;
            let y: u16 = 100;
            let z: u8 = 7;

            // Minimal InitGame
            let mut p = Vec::new();
            p.push(0x0A);
            p.extend(&player_id.to_le_bytes());
            send(stream, &p, "InitGame (minimal)");

            // Immediately send map
            let mut map = Vec::new();
            map.push(0x64);
            map.extend(&x.to_le_bytes());
            map.extend(&y.to_le_bytes());
            map.push(z);
            // 252 tiles with item ID 1 (usually first item in dat)
            for _ in 0..252 {
                map.extend(&1u16.to_le_bytes());  // item ID 1
                map.push(0xFF);
                map.push(0xFF);
            }
            send(stream, &map, "FullMap (item ID 1)");
        }

        12 => {
            println!("=== VARIANT 12: Different player ID format ===");
            // Maybe player ID should be smaller or in different format?
            let small_id: u32 = 1;
            let mut p = Vec::new();
            p.push(0x0A);
            p.extend(&small_id.to_le_bytes());
            p.extend(&0u16.to_le_bytes());
            p.push(0);
            p.extend(&0u16.to_le_bytes());
            send(stream, &p, "InitGame (playerId=1)");
        }

        _ => {
            println!("Unknown variant {}. Use 1-12.", variant);
        }
    }

    println!("");
}

fn send(stream: &mut TcpStream, body: &[u8], name: &str) {
    let len = body.len() as u16;
    let mut pkt = len.to_le_bytes().to_vec();
    pkt.extend(body);

    print!("  {}: ", name);
    if stream.write_all(&pkt).is_ok() {
        let _ = stream.flush();
        if body.len() <= 30 {
            println!("{:02X?}", body);
        } else {
            println!("{:02X?}... ({} bytes)", &body[..20], body.len());
        }
    } else {
        println!("WRITE ERROR");
    }
    thread::sleep(Duration::from_millis(50));
}

fn monitor_client(stream: &mut TcpStream) {
    println!("[WAITING] Monitoring for client response...\n");

    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > Duration::from_secs(10) {
            println!("[TIMEOUT] No disconnect after 10s - client might be waiting for more data!");
            break;
        }

        let mut len_buf = [0u8; 2];
        match stream.read_exact(&mut len_buf) {
            Ok(_) => {
                let length = u16::from_le_bytes(len_buf) as usize;
                if length > 0 && length < 10000 {
                    let mut body = vec![0u8; length];
                    if stream.read_exact(&mut body).is_ok() {
                        let opcode = body.first().copied().unwrap_or(0);
                        println!("[RECV] opcode=0x{:02X} len={} → Client sent something!", opcode, length);
                        println!("       This means the init packet was accepted!");
                        println!("       Body: {:02X?}", &body[..body.len().min(30)]);
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
                println!("[DISCONNECTED] Client rejected the packet(s)");
                break;
            }
        }
    }
}