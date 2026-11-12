// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand};
use smc_lib::io::IOService;

#[derive(Parser)]
#[command(
    name = "mac-led-control",
    about = "Control the front status LED on a Mac mini"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Turn the LED on
    On,
    /// Turn the LED off
    Off,
}

fn main() {
    let cli = Cli::parse();

    let (value, name): (u8, &str) = match cli.command {
        Commands::On => (0x01, "on"),
        Commands::Off => (0x00, "off"),
    };

    let smc = match IOService::init() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error connecting to SMC: {e}");
            eprintln!("Hint: this command requires root. Try `sudo mac-led-control {name}`.");
            std::process::exit(1);
        }
    };

    match smc.write_key(b"LSOO", &[value]) {
        Ok(_) => println!("LED turned {name}."),
        Err(e) => {
            eprintln!("Failed to write LSOO: {e}");
            eprintln!("Hint: this command requires root. Try `sudo mac-led-control {name}`.");
            std::process::exit(1);
        }
    }
}
