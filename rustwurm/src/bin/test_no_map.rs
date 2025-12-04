//! Test server WITHOUT map data
//!
//! This version skips the FullMap packet to see if other packets are accepted.
//! If the client still hangs, the issue is in InitGame/PlayerData packets.
//! If the client shows an error or disconnects, then it requires map data.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::thread;

fn main() {
    println!("=== Tibia 1.03 Test Server (NO MAP) ===");
    println!("Listening on 127.0.0.1:7171\n");

    let listener = TcpListener::bind("127.0.0.1:7171").expect("Failed to bind");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("[CONNECT] {:?}", stream.peer_addr());
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

                if !read_login(&mut stream) { continue; }
                send_init_no_map(&mut stream);
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
        println!("[LOGIN] {}", name);
    }
    true
}

fn send_init_no_map(stream: &mut TcpStream) {
    let player_id: u32 = 0x10000001;
    let x: u16 = 100;
    let y: u16 = 100;
    let z: u8 = 7;

    println!("\n[SENDING] Init sequence WITHOUT map:\n");

    // InitGame (0x0A)
    let mut p = Vec::new();
    p.push(0x0A);
    p.extend(&player_id.to_le_bytes());
    p.extend(&0u16.to_le_bytes());  // sessionFlags
    p.push(0);                       // canReportBugs
    p.extend(&0u16.to_le_bytes());  // welcomeText (empty)
    send(stream, &p, "InitGame");

    // PlayerDataBasic (0x9F)
    let mut p = Vec::new();
    p.push(0x9F);
    p.extend(&player_id.to_le_bytes());
    p.extend(&x.to_le_bytes());
    p.extend(&y.to_le_bytes());
    p.push(z);
    send(stream, &p, "PlayerDataBasic");

    // PlayerData (0xA0)
    let mut p = Vec::new();
    p.push(0xA0);
    p.extend(&100u16.to_le_bytes()); // hp
    p.extend(&100u16.to_le_bytes()); // maxHp
    p.extend(&400u16.to_le_bytes()); // capacity
    p.extend(&0u32.to_le_bytes());   // exp
    p.extend(&1u16.to_le_bytes());   // level
    p.extend(&50u16.to_le_bytes());  // mana
    p.extend(&50u16.to_le_bytes());  // maxMana
    p.push(0);                        // magicLvl
    p.push(0);                        // magicLvl%
    send(stream, &p, "PlayerData");

    // PlayerSkills (0xA1)
    let mut p = Vec::new();
    p.push(0xA1);
    for _ in 0..7 { p.push(10); p.push(0); }
    send(stream, &p, "PlayerSkills");

    // SKIP MAP!
    println!("  [SKIPPING MAP]\n");

    // TextMessage (0xB4)
    let msg = "Welcome! (no map sent)";
    let mut p = Vec::new();
    p.push(0xB4);
    p.push(0x12);
    p.extend(&(msg.len() as u16).to_le_bytes());
    p.extend(msg.as_bytes());
    send(stream, &p, "TextMessage");

    println!("");
}

fn send(stream: &mut TcpStream, body: &[u8], name: &str) {
    let len = body.len() as u16;
    let mut pkt = len.to_le_bytes().to_vec();
    pkt.extend(body);
    print!("  {}: ", name);
    if stream.write_all(&pkt).is_ok() {
        let _ = stream.flush();
        println!("{:02X?}", &body[..body.len().min(20)]);
    } else {
        println!("ERROR");
    }
    thread::sleep(Duration::from_millis(5));
}

fn monitor_client(stream: &mut TcpStream) {
    println!("[WAITING for client response...]\n");
    loop {
        let mut len_buf = [0u8; 2];
        match stream.read_exact(&mut len_buf) {
            Ok(_) => {
                let length = u16::from_le_bytes(len_buf) as usize;
                if length > 0 && length < 10000 {
                    let mut body = vec![0u8; length];
                    if stream.read_exact(&mut body).is_ok() {
                        println!("[RECV] op=0x{:02X} len={}", body[0], length);
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(_) => { println!("[DISCONNECTED]"); break; }
        }
    }
}