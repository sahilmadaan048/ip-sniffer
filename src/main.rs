use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::process;
use std::str::FromStr; 
use std::sync::mpsc::{channel, Sender};
use std::thread;

const MAX: u16 = 65535;

struct Arguments {
    ipaddr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("Not enough arguments. Use -h or -help for usage instructions.");
        } else if args.len() > 4 {
            return Err("Too many arguments.");
        }

        if args[1] == "-h" || args[1] == "-help" {
            println!(
                "Usage: 
                \n  ip_snipper.exe -h or -help            Show this message
                \n  ip_snipper.exe <IP>                   Scan using default 4 threads
                \n  ip_snipper.exe -j <num_threads> <IP>   Scan using a custom number of threads"
            );
            return Err("help");
        }

        if args[1] == "-j" {
            if args.len() != 4 {
                return Err("Usage: -j <num_threads> <IP>");
            }
            let threads = args[2].parse::<u16>().map_err(|_| "Invalid thread count")?;
            let ipaddr = IpAddr::from_str(&args[3]).map_err(|_| "Invalid IP address")?;
            return Ok(Arguments { ipaddr, threads });
        }

        let ipaddr = IpAddr::from_str(&args[1]).map_err(|_| "Invalid IP address")?;
        Ok(Arguments { ipaddr, threads: 4 }) // Default to 4 threads
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port = start_port + 1;
    while port <= MAX {
        if TcpStream::connect((addr, port)).is_ok() {
            print!(".");
            io::stdout().flush().unwrap();
            tx.send(port).unwrap();
        }
        port += num_threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err == "help" {
            process::exit(0);
        } else {
            eprintln!("{}: {}", program, err);
            process::exit(1);
        }
    });

    let num_threads = arguments.threads;
    let addr = arguments.ipaddr;
    let (tx, rx) = channel();

    for i in 0..num_threads {
        let tx = tx.clone();
        thread::spawn(move || {
            scan(tx, i, addr, num_threads);
        });
    }

    let mut out = Vec::new();
    drop(tx); // Ensure no sender remains

    for port in rx {
        out.push(port);
    }

    println!("\nScan complete!");
    out.sort();
    for port in out {
        println!("Port {} is open", port);
    }
}


// ip_snipper.exe -h   => open up a help screen
// ip_snipper.exe -j 100 192.168.1.1  => set how many threads users want to use on this process
// ip_snipper.exe 192.168.1.1  => calling the tool on a ip address