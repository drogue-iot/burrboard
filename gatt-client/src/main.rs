use std::path::PathBuf;

use bluer::{AdapterEvent, Address};
use clap::{ArgEnum, Parser};
use core::str::FromStr;
use futures::{pin_mut, StreamExt};
use log;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

mod board;
mod firmware;
use crate::board::{BurrBoard, Led};
use crate::firmware::*;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    device: String,

    #[clap(short, long)]
    interval: Option<u16>,

    #[clap(long)]
    turn_on: Option<Led>,

    #[clap(long)]
    turn_off: Option<Led>,

    #[clap(long)]
    firmware: Option<PathBuf>,

    #[clap(long)]
    mode: Mode,

    #[clap(short, long)]
    verbosity: Option<usize>,

    #[]
}

#[derive(Debug, ArgEnum, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Gateway,
    Client,
}

fn merge(a: &mut serde_json::Value, b: &serde_json::Value) {
    match (a, b) {
        (&mut serde_json::Value::Object(ref mut a), &serde_json::Value::Object(ref b)) => {
            for (k, v) in b {
                merge(a.entry(k.clone()).or_insert(serde_json::Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    stderrlog::new()
        .verbosity(args.verbosity.unwrap_or(0))
        .init()
        .unwrap();

    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    let discover = adapter.discover_devices().await?;
    pin_mut!(discover);

    let addr = Address::from_str(&args.device)?;
    let mut firmware_updated = false;

    while let Some(evt) = discover.next().await {
        match evt {
            AdapterEvent::DeviceAdded(a) if a == addr => {
                let device = adapter.device(a)?;

                sleep(Duration::from_secs(2)).await;
                if !device.is_connected().await? {
                    println!("Connecting...");
                    let mut retries = 2;
                    loop {
                        match device.connect().await {
                            Ok(()) => break,
                            Err(err) if retries > 0 => {
                                println!("Connect error: {}", &err);
                                retries -= 1;
                            }
                            Err(err) => return Err(err)?,
                        }
                    }
                    println!("Connected");
                } else {
                    println!("Already connected");
                }
                let board = BurrBoard::new(device);
                let version = board.read_firmware_version().await?;
                println!("Connected to board! Running version {}", version);
                if
                match args.operation {
                    Operation::Server => {
                        let s = board.stream_sensors().await?;
                        pin_mut!(s);
                        let mut view = json!({});
                        while let Some(n) = s.next().await {
                            merge(&mut view, &n);
                            println!("{}", view);
                        }
                    }
                    Operation::Read => {
                        let sensor = board.read_sensors().await?;
                        println!("{}", sensor);
                        return Ok(());
                    }
                    Operation::Write => {
                        if let Some(i) = args.interval {
                            board.set_interval(i).await?;
                            return Ok(());
                        }
                        if let Some(led) = args.turn_on {
                            board.set_led(led, true).await?;
                            return Ok(());
                        }
                        if let Some(led) = args.turn_off {
                            board.set_led(led, false).await?;
                            return Ok(());
                        }
                        if let Some(firmware) = &args.firmware {
                            let metadata = FirmwareMetadata::from_file(firmware)?;
                            if !firmware_updated {
                                println!(
                                    "Updating firmware from version {} to {}",
                                    version, &metadata.version
                                );
                                match metadata.data {
                                    FirmwareData::Bytes(_) => {
                                        todo!()
                                    }
                                    FirmwareData::File(path) => {
                                        board.update_firmware(&path).await?;
                                        firmware_updated = true;
                                        adapter.remove_device(board.free().address()).await?;
                                        println!("Firmware is updated. Waiting for device to come back online...");
                                    }
                                }
                            } else {
                                if version != metadata.version {
                                    return Err(anyhow::anyhow!(
                                    "Error during firmware update! Device reports {}, expected {}",
                                    version,
                                    metadata.version
                                ));
                                } else {
                                    // Confirm that firmware is now using the latest version and mark it as bootable
                                    println!("Firmware updated successfully");
                                    board.mark_booted().await?;
                                    println!("Device firmware marked as booted");
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
            AdapterEvent::DeviceRemoved(a) if a == addr => {
                log::info!("Device removed: {}", a);
            }
            _ => {}
        }
    }

    log::info!("BLE sensor disconnected, shutting down");
    Ok(())
}
