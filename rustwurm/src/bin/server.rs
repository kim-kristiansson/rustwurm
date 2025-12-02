use rustwurm::server_core::run_server;
use rustwurm::protocol::v1_03::Protocol;

fn main() {
    if let Err(e) = run_server::<Protocol>(){
        println!("Error: {}", e);
    }
}