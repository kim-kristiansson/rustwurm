use rustwurm::server_core::run_server;
use rustwurm::protocol::tibia103::Protocol;

fn main() {
    if let Err(e) = run_server::<Protocol>(){
        println!("Error: {}", e);
    }
}