use anyhow::anyhow;
use burrboard_dfu::*;
use clap::{Parser, Subcommand};
use futures::io::AllowStdIo;
use futures::pin_mut;
use postcard::{from_bytes, to_allocvec};
use serial::SerialPort;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: ToolCommand,

    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolCommand {
    Update {
        #[clap(subcommand)]
        transport: Transport,

        #[clap(short, long)]
        firmware: PathBuf,
    },
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Transport {
    Serial {
        #[clap(short, long)]
        port: PathBuf,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    stderrlog::new().verbosity(args.verbose).init().unwrap();

    match &args.command {
        ToolCommand::Update {
            transport,
            firmware,
        } => match transport {
            Transport::Serial { port } => {
                let mut port = serial::open(&port).expect("Unable to open serial port");
                port.reconfigure(&|settings| {
                    settings.set_baud_rate(serial::Baud115200)?;
                    settings.set_char_size(serial::Bits8);
                    settings.set_parity(serial::ParityNone);
                    settings.set_stop_bits(serial::Stop1);
                    settings.set_flow_control(serial::FlowNone);
                    Ok(())
                })?;
                port.set_timeout(Duration::from_millis(1000))?;

                update_firmware_serial(&mut port, firmware)?;
            }
        },
    }
    Ok(())
}

fn update_firmware_serial<T: SerialPort>(
    port: &mut T,
    firmware: &PathBuf,
) -> Result<(), anyhow::Error> {
    let mut f = File::open(firmware)?;

    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer)?;

    log::info!("Starting DFU process");
    Frame::Command(Command::Version)
        .encode_blocking(port)
        .map_err(|_| anyhow!("Error sending command"))?;

    let r = Frame::decode_blocking(port).map_err(|_| anyhow!("Error decoding command"))?;

    log::info!("Received response: {:?}", r);

    /*

    send_command(port, Command::Start, None)?;
    log::info!("Signalled start");
    send_command(port, Command::Write(buffer.len() as u32), Some(&buffer[..]))?;
    log::info!("Write done");
    send_command(port, Command::Finish, None)?;
    log::info!("Write done");
    */

    Ok(())
}
