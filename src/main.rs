use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::{self, Write};
use std::net::UdpSocket;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about = "Rust port of IceCon with timeouts and retries")]
struct Args {
    /// Server address (IP:port)
    address: String,

    /// RCON password
    password: String,

    /// Command to run (if omitted, enters interactive shell)
    #[arg(short, long)]
    command: Option<String>,

    /// Timeout in seconds for each request
    #[arg(short, long, default_value = "2")]
    timeout: u64,

    /// Number of retries on timeout
    #[arg(short, long, default_value = "3")]
    retries: u32,
}

const OOB_HEADER: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(args.timeout)))?;

    if let Some(cmd) = args.command {
        let response = send_with_retry(&socket, &args.address, &args.password, &cmd, args.retries)?;
        println!("{}", response);
    } else {
        run_shell(socket, args.address, args.password, args.retries)?;
    }

    Ok(())
}

fn send_with_retry(
    socket: &UdpSocket,
    addr: &str,
    password: &str,
    command: &str,
    max_retries: u32,
) -> Result<String> {
    let mut last_err = anyhow!("Failed after max retries");
    
    for attempt in 0..=max_retries {
        if attempt > 0 {
            eprintln!("Retrying... (attempt {}/{})", attempt, max_retries);
        }

        match send_and_receive(socket, addr, password, command) {
            Ok(response) => return Ok(response),
            Err(e) => {
                last_err = e;
                // If it's a timeout error, we continue to retry
                // Otherwise, we might want to fail fast (e.g. invalid address)
            }
        }
    }

    Err(last_err)
}

fn send_and_receive(socket: &UdpSocket, addr: &str, password: &str, command: &str) -> Result<String> {
    // Construct packet: [OOB] rcon [password] [command]\n
    let mut packet = Vec::new();
    packet.extend_from_slice(&OOB_HEADER);
    packet.extend_from_slice(format!("rcon {} {}\n", password, command).as_bytes());

    socket.send_to(&packet, addr)?;

    let mut buf = [0u8; 65535];
    let (amt, _) = socket.recv_from(&mut buf)?;

    let data = &buf[..amt];
    if data.len() < 4 || data[..4] != OOB_HEADER {
        return Err(anyhow!("Invalid response: missing OOB header"));
    }

    let body = String::from_utf8_lossy(&data[4..]);
    
    // Server response is usually "print\n[text]"
    if body.starts_with("print\n") {
        Ok(body[6..].to_string())
    } else if body.starts_with("print") {
         // Some variants might not have the newline immediately after print
        Ok(body[5..].trim_start().to_string())
    } else {
        Ok(body.to_string())
    }
}

fn run_shell(socket: UdpSocket, addr: String, password: String, retries: u32) -> Result<()> {
    println!("QRCON Shell - Type 'quit' to exit");
    let mut input = String::new();
    let stdin = io::stdin();

    loop {
        print!("{}> ", addr);
        io::stdout().flush()?;
        
        input.clear();
        stdin.read_line(&mut input)?;
        let cmd = input.trim();

        if cmd.is_empty() {
            continue;
        }
        if cmd == "quit" || cmd == "exit" {
            break;
        }

        match send_with_retry(&socket, &addr, &password, cmd, retries) {
            Ok(resp) => println!("{}", resp),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
