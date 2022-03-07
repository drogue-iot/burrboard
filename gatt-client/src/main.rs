use std::path::PathBuf;

use bluer::{AdapterEvent, Address};
use clap::{Parser, Subcommand};
use core::str::FromStr;
use futures::{pin_mut, StreamExt};
use log;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

mod board;
mod firmware;
#[cfg(feature = "hawkbit")]
mod hawkbit;

use crate::board::{BurrBoard, Led};
use crate::firmware::*;
#[cfg(feature = "hawkbit")]
use crate::hawkbit::*;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    mode: Mode,

    #[clap(short, long)]
    device: String,

    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Gateway {
        #[clap(long)]
        endpoint_url: String,

        #[clap(long)]
        endpoint_user: String,

        #[clap(long)]
        endpoint_password: String,

        #[clap(long)]
        firmware_url: Option<String>,
    },
    Client {
        #[clap(short, long)]
        read: bool,

        #[clap(short, long)]
        stream: bool,

        #[clap(long)]
        turn_on: Option<Led>,

        #[clap(long)]
        turn_off: Option<Led>,

        #[clap(long)]
        firmware: Option<PathBuf>,

        #[clap(long)]
        report_interval: Option<u16>,
    },
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
    stderrlog::new().verbosity(args.verbose).init().unwrap();

    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    let discover = adapter.discover_devices().await?;
    pin_mut!(discover);

    let addr = Address::from_str(&args.device)?;
    let mut firmware_updated = false;
    let mut deployment: Option<Deployment> = None;

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
                match &args.mode {
                    Mode::Client {
                        read,
                        stream,
                        turn_on,
                        turn_off,
                        firmware,
                        report_interval,
                    } => {
                        if *read {
                            let sensor = board.read_sensors().await?;
                            println!("{}", sensor);
                            return Ok(());
                        }
                        if *stream {
                            let s = board.stream_sensors().await?;
                            pin_mut!(s);
                            let mut view = json!({});
                            while let Some(n) = s.next().await {
                                let previous = view.clone();
                                merge(&mut view, &n);
                                if previous != view {
                                    println!("{}", view);
                                }
                            }
                        }
                        if let Some(i) = report_interval {
                            board.set_interval(*i).await?;
                            return Ok(());
                        }
                        if let Some(led) = turn_on {
                            board.set_led(*led, true).await?;
                            return Ok(());
                        }
                        if let Some(led) = turn_off {
                            board.set_led(*led, false).await?;
                            return Ok(());
                        }
                        if let Some(firmware) = firmware {
                            let metadata = FirmwareMetadata::from_file(firmware)?;
                            if !firmware_updated {
                                println!(
                                    "Updating firmware from version {} to {}",
                                    version, &metadata.version
                                );
                                match metadata.data {
                                    FirmwareData::Http(_) => {
                                        todo!()
                                    }
                                    FirmwareData::File(path) => {
                                        board.update_firmware_from_file(&path).await?;
                                        firmware_updated = true;
                                        adapter.remove_device(board.address()).await?;
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
                    Mode::Gateway {
                        endpoint_url,
                        endpoint_user,
                        endpoint_password,
                        firmware_url,
                    } => {
                        let firmware_client = firmware_url.as_deref().map(FirmwareClient::new);

                        // We need to finish an earlier deployment
                        if let Some(deployment) = deployment.take() {
                            let metadata = &deployment.metadata;
                            if version != metadata.version {
                                println!(
                                    "Error during firmware update! Device reports {}, expected {}",
                                    version, metadata.version
                                );
                            } else {
                                // Confirm that firmware is now using the latest version and mark it as bootable
                                println!("Firmware updated successfully");
                                board.mark_booted().await?;
                                println!("Device firmware marked as booted");
                            }
                        }

                        let board = Arc::new(board);
                        // Stream sensors
                        let endpoint_url = endpoint_url.to_string();
                        let endpoint_user = endpoint_user.to_string();
                        let endpoint_password = endpoint_password.to_string();
                        let b = board.clone();
                        let stream_task = tokio::task::spawn(async move {
                            log::info!("Running data stream for '{a}'");
                            let mut s = Box::pin(b.stream_sensors().await.unwrap());
                            let client = reqwest::Client::new();
                            let mut view = json!({});
                            loop {
                                if let Some(n) = s.next().await {
                                    let previous = view.clone();
                                    merge(&mut view, &n);
                                    if previous != view {
                                        let payload = json! {
                                            {
                                                "features": view,
                                            }
                                        };
                                        log::debug!("Payload: {payload}");
                                        match client
                                            .post(&endpoint_url)
                                            .basic_auth(&endpoint_user, Some(&endpoint_password))
                                            .json(&payload)
                                            .send()
                                            .await
                                        {
                                            Ok(resp) if !resp.status().is_success() => {
                                                println!(
                                                    "Error response {}: {}",
                                                    resp.status(),
                                                    resp.text().await.unwrap_or_default()
                                                );
                                            }
                                            Ok(_) => {}
                                            Err(e) => {
                                                println!("Request error: {:?}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        });

                        // Wait for deployment
                        if let Some(firmware_client) = firmware_client {
                            let d = firmware_client.wait_update(&version).await?;
                            let metadata = &d.metadata;
                            println!(
                                "Updating firmware from version {} to {}",
                                version, &metadata.version
                            );

                            match &metadata.data {
                                FirmwareData::Http(url) => {
                                    // Download file
                                    let data = firmware_client.fetch_firmware(url).await?;
                                    println!("Received firmware of {} bytes", data.len());
                                    board.update_firmware(&data[..]).await?;
                                    stream_task.abort();
                                    deployment.replace(d);
                                    adapter.remove_device(board.address()).await?;
                                    println!("Firmware is updated. Waiting for device to come back online...");
                                }
                                FirmwareData::File(_path) => {
                                    panic!("unexpected metadata");
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
