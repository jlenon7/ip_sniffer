use std::{
    env,
    io::{self, Write},
    net::{IpAddr, TcpStream},
    process,
    str::FromStr,
    sync::mpsc::{channel, Sender},
    thread,
};

const MAX: u16 = 65535;

struct Arguments {
    ipaddr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("not enought arguments");
        }

        if args.len() > 4 {
            return Err("too many arguments");
        }

        let flag = args[1].clone();

        if let Ok(ipaddr) = IpAddr::from_str(&flag) {
            return Ok(Arguments { ipaddr, threads: 4 });
        }

        if flag.contains("-h") || flag.contains("--help") {
            if args.len() > 2 {
                return Err("too many arguments");
            }

            println!(
                "Usage -j to select how many threads you want
            \r\n      -h or --help to show help message"
            );
            process::exit(0);
        }

        if flag.contains("-j") {
            let ipaddr = match IpAddr::from_str(&args[3]) {
                Ok(s) => s,
                Err(_) => return Err("not valid IPADDR; must be iPv4 or iPv6"),
            };

            let threads = match args[2].parse::<u16>() {
                Ok(s) => s,
                Err(_) => return Err("failed to parse thread number"),
            };

            return Ok(Arguments { threads, ipaddr });
        } else {
            return Err("invalid syntax");
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, threads: u16) {
    let mut port: u16 = start_port + 1;

    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        };

        if (MAX - port) <= threads {
            break;
        }

        port += threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        eprintln!("{} problem parsing arguments: {}", program, err);
        process::exit(1);
    });

    let (tx, rx) = channel();

    for i in 0..arguments.threads {
        let tx = tx.clone();

        thread::spawn(move || {
            scan(tx, i, arguments.ipaddr, arguments.threads);
        });
    }

    let mut out = vec![];
    drop(tx);

    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();

    for v in out {
        println!("{} is open", v);
    }
}
