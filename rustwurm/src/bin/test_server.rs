//! Ultra-minimal Tibia 1.03 test server
//!
//! Tests packet formats one by one to find what the client accepts.
//!
//! To use: Save as src/bin/test_server.rs and run with:
//!   cargo run --bin test_server

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::thread;

fn main() {
    println!("=== Tibia 1.03 Minimal Test Server ===");
    println!("Listening on 127.0.0.1:7171\n");

    let listener = TcpListener::bind("127.0.0.1:7171").expect("Failed to bind");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("[CONNECT] {:?}", stream.peer_addr());

                // Set read timeout so we can detect disconnects
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

                if !read_login(&mut stream) {
                    continue;
                }

                // Test different init sequences
                test_init_sequence(&mut stream);

                // Keep reading to see what client sends
                monitor_client(&mut stream);
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }
}

fn read_login(stream: &mut TcpStream) -> bool {
    let mut len_buf = [0u8; 2];
    if stream.read_exact(&mut len_buf).is_err() {
        println!("[ERROR] Failed to read login packet");
        return false;
    }
    let length = u16::from_le_bytes(len_buf) as usize;

    let mut body = vec![0u8; length];
    if stream.read_exact(&mut body).is_err() {
        println!("[ERROR] Failed to read login body");
        return false;
    }

    println!("[RECV LOGIN] {} bytes", length);
    println!("  First 20 bytes: {:02X?}", &body[..20.min(body.len())]);

    // Extract name
    if body.len() >= 37 {
        let name_bytes = &body[7..37];
        let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(30);
        let name = String::from_utf8_lossy(&name_bytes[..name_end]);
        println!("  Player name: {}", name);
    }

    true
}

fn test_init_sequence(stream: &mut TcpStream) {
    let player_id: u32 = 0x10000001;
    let x: u16 = 32097; // Use typical Tibia coordinates
    let y: u16 = 32219;
    let z: u8 = 7;

    println!("\n[SENDING INIT SEQUENCE]");

    // ════════════════════════════════════════════════════════════════════
    // Packet 1: InitGame (0x0A)
    // ════════════════════════════════════════════════════════════════════
    let mut init_game = Vec::new();
    init_game.push(0x0A);                                    // opcode
    init_game.extend_from_slice(&player_id.to_le_bytes());   // playerId
    init_game.extend_from_slice(&0u16.to_le_bytes());        // sessionFlags
    init_game.push(0x00);                                    // canReportBugs
    init_game.extend_from_slice(&0u16.to_le_bytes());        // welcomeText len=0
    send(stream, &init_game, "InitGame (0x0A)");

    // ════════════════════════════════════════════════════════════════════
    // Packet 2: PlayerDataBasic (0x9F)
    // ════════════════════════════════════════════════════════════════════
    let mut pdb = Vec::new();
    pdb.push(0x9F);
    pdb.extend_from_slice(&player_id.to_le_bytes());
    pdb.extend_from_slice(&x.to_le_bytes());
    pdb.extend_from_slice(&y.to_le_bytes());
    pdb.push(z);
    send(stream, &pdb, "PlayerDataBasic (0x9F)");

    // ════════════════════════════════════════════════════════════════════
    // Packet 3: PlayerData (0xA0)
    // ════════════════════════════════════════════════════════════════════
    let mut pd = Vec::new();
    pd.push(0xA0);
    pd.extend_from_slice(&100u16.to_le_bytes());  // hp
    pd.extend_from_slice(&100u16.to_le_bytes());  // maxHp
    pd.extend_from_slice(&400u16.to_le_bytes());  // capacity
    pd.extend_from_slice(&0u32.to_le_bytes());    // experience
    pd.extend_from_slice(&1u16.to_le_bytes());    // level
    pd.extend_from_slice(&50u16.to_le_bytes());   // mana
    pd.extend_from_slice(&50u16.to_le_bytes());   // maxMana
    pd.push(0);                                    // magicLevel
    pd.push(0);                                    // magicLevel%
    send(stream, &pd, "PlayerData (0xA0)");

    // ════════════════════════════════════════════════════════════════════
    // Packet 4: PlayerSkills (0xA1)
    // ════════════════════════════════════════════════════════════════════
    let mut ps = Vec::new();
    ps.push(0xA1);
    for _ in 0..7 {
        ps.push(10);  // skill level
        ps.push(0);   // skill %
    }
    send(stream, &ps, "PlayerSkills (0xA1)");

    // ════════════════════════════════════════════════════════════════════
    // Packet 5: FullMap (0x64) - Minimal version
    // ════════════════════════════════════════════════════════════════════
    // This is the critical one. Let's try a minimal map.
    //
    // Format (from protocol spec):
    // - Center position (x:u16, y:u16, z:u8)
    // - Then 18*14 = 252 tiles
    // - Each tile: items until 0xFF 0xFF terminator
    //
    // For Tibia 1.03, we need REAL item IDs from the client's tibia.dat
    // Common ground tiles in early Tibia were around ID 100-500
    // Let's try item ID 102 (often grass in old clients)

    let mut map = Vec::new();
    map.push(0x64);  // opcode

    // Center position
    map.extend_from_slice(&x.to_le_bytes());
    map.extend_from_slice(&y.to_le_bytes());
    map.push(z);

    // 18x14 = 252 tiles
    // Each tile: ground item + terminator
    // Using item ID 102 (common grass tile in old Tibia)
    let ground_item: u16 = 102;

    for _ in 0..(18 * 14) {
        map.extend_from_slice(&ground_item.to_le_bytes());  // ground item
        map.push(0xFF);  // terminator part 1
        map.push(0xFF);  // terminator part 2
    }

    // Map terminator (might be needed!)
    map.push(0xFE);
    map.push(0x00);

    send(stream, &map, "FullMap (0x64)");

    // ════════════════════════════════════════════════════════════════════
    // Packet 6: TextMessage (0xB4)
    // ════════════════════════════════════════════════════════════════════
    let msg = "Welcome to Rustwurm!";
    let mut tm = Vec::new();
    tm.push(0xB4);
    tm.push(0x12);  // message type (info)
    tm.extend_from_slice(&(msg.len() as u16).to_le_bytes());
    tm.extend_from_slice(msg.as_bytes());
    send(stream, &tm, "TextMessage (0xB4)");

    println!("\n[INIT COMPLETE]\n");
}

fn send(stream: &mut TcpStream, body: &[u8], name: &str) {
    let len = body.len() as u16;
    let mut packet = len.to_le_bytes().to_vec();
    packet.extend_from_slice(body);

    print!("  {} ({} bytes): ", name, body.len());

    match stream.write_all(&packet) {
        Ok(_) => {
            let _ = stream.flush();
            if body.len() <= 30 {
                println!("{:02X?}", body);
            } else {
                println!("{:02X?}...", &body[..30]);
            }
        }
        Err(e) => println!("ERROR: {}", e),
    }

    // Small delay between packets
    thread::sleep(Duration::from_millis(10));
}

fn monitor_client(stream: &mut TcpStream) {
    println!("[MONITORING] Waiting for client packets (5s timeout)...\n");

    loop {
        let mut len_buf = [0u8; 2];
        match stream.read_exact(&mut len_buf) {
            Ok(_) => {
                let length = u16::from_le_bytes(len_buf) as usize;
                if length > 0 && length < 10000 {
                    let mut body = vec![0u8; length];
                    if stream.read_exact(&mut body).is_ok() {
                        let opcode = body.first().copied().unwrap_or(0);
                        println!("[RECV] opcode=0x{:02X} len={} body={:02X?}",
                                 opcode, length, &body[..body.len().min(50)]);

                        // Decode known opcodes
                        match opcode {
                            0x65 => println!("       → Walk North"),
                            0x66 => println!("       → Walk East"),
                            0x67 => println!("       → Walk South"),
                            0x68 => println!("       → Walk West"),
                            0x6F => println!("       → Turn North"),
                            0x70 => println!("       → Turn East"),
                            0x71 => println!("       → Turn South"),
                            0x72 => println!("       → Turn West"),
                            0x96 => println!("       → Say"),
                            0xA1 => println!("       → Attack"),
                            0xBE => println!("       → Cancel"),
                            _ => {}
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Timeout - that's fine
                thread::sleep(Duration::from_millis(100));
            }
            Err(_) => {
                println!("[DISCONNECTED]");
                break;
            }
        }
    }
}