mod color;
mod processor;
mod state;

use clap::Parser;
use colored::Color;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use color::parse_color;
use processor::LineProcessor;
use state::{step, Action, Event, LogState};

#[derive(Parser)]
#[command(name = "tail-divider")]
#[command(about = "A CLI tool for tail log file divider")]
#[command(version)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[arg(
        short = 't',
        long,
        default_value = "3",
        help = "Idle threshold in seconds"
    )]
    idle: u64,

    #[arg(short, long, help = "Idle message color (e.g., #FF6600, red, green)")]
    color: Option<String>,

    #[arg(short, long, value_name = "FILE", help = "Follow a file (like tail -f)")]
    follow: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let threshold = Duration::from_secs(cli.idle);

    let idle_color = match cli.color.as_deref().map(parse_color) {
        Some(Ok(c)) => c,
        Some(Err(e)) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        None => Color::TrueColor { r: 128, g: 128, b: 128 },
    };

    if cli.verbose {
        eprintln!("Verbose mode enabled");
        eprintln!("Idle threshold: {} seconds", cli.idle);
    }

    let processor = LineProcessor::new(idle_color);
    let (tx, rx) = channel::<String>();

    match cli.follow {
        Some(path) => {
            thread::spawn(move || {
                let file = match File::open(&path) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("error: cannot open '{}': {}", path, e);
                        std::process::exit(1);
                    }
                };
                let mut reader = BufReader::new(file);
                reader.seek(SeekFrom::End(0)).unwrap();
                let mut line = String::new();
                loop {
                    match reader.read_line(&mut line) {
                        Ok(0) => std::thread::sleep(Duration::from_millis(100)),
                        Ok(_) => {
                            let trimmed = line.trim_end_matches('\n')
                                .trim_end_matches('\r')
                                .to_string();
                            line.clear();
                            if tx.send(trimmed).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }
        None => {
            thread::spawn(move || {
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    let line = line.unwrap();
                    if tx.send(line).is_err() {
                        break;
                    }
                }
            });
        }
    };

    let mut state = LogState::Active {
        last_activity: Instant::now(),
        idle_count: 0,
    };

    loop {
        let event = match &state {
            LogState::Active { last_activity, .. } => {
                let remaining = threshold.saturating_sub(last_activity.elapsed());
                match rx.recv_timeout(remaining) {
                    Ok(line) => Event::Line(line),
                    Err(RecvTimeoutError::Timeout) => Event::Timeout,
                    Err(RecvTimeoutError::Disconnected) => Event::Disconnected,
                }
            }
            LogState::Idle { .. } => match rx.recv() {
                Ok(line) => Event::Line(line),
                Err(_) => Event::Disconnected,
            },
        };

        let (next_state, actions) = step(state, event, &processor);
        state = next_state;

        let mut should_break = false;
        for action in actions {
            match action {
                Action::Print(s) => {
                    println!("{}", s);
                    io::stdout().flush().unwrap();
                }
                Action::PrintBlankLine => println!(),
                Action::Break => should_break = true,
                Action::Debug(msg) => {
                    if cli.verbose {
                        eprintln!("[DEBUG] {}", msg);
                    }
                }
            }
        }
        if should_break {
            break;
        }
    }
}
