use clap::{Parser, Subcommand};
use drgdfu::{FirmwareDevice, GattBoard};
use futures::lock::Mutex;
use futures::{pin_mut, StreamExt};
use serde_json::json;
use std::process::exit;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

mod board;
mod firmware;
mod gateway;

use crate::board::{BurrBoard, Led};
use crate::firmware::*;
use crate::gateway::*;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    mode: Mode,

    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    #[clap(short, long, parse(try_from_str=humantime::parse_duration))]
    timeout: Option<Duration>,
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Gateway {
        #[clap(long)]
        http: String,

        #[clap(long)]
        user: String,

        #[clap(long)]
        password: String,

        #[clap(short, long)]
        devices: String,

        #[clap(long)]
        enable_dfu: bool,
    },
    Client {
        #[clap(short, long)]
        read: bool,

        #[clap(short, long)]
        device: String,

        #[clap(short, long)]
        stream: bool,

        #[clap(long)]
        turn_on: Option<Led>,

        #[clap(long)]
        turn_off: Option<Led>,

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
    let adapter = Arc::new(session.default_adapter().await?);
    adapter.set_powered(true).await?;

    let last_event = Arc::new(Mutex::new(Instant::now()));

    if let Some(timeout) = args.timeout {
        let last_event = last_event.clone();

        tokio::task::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;

                if Instant::now() - *last_event.lock().await > timeout {
                    log::error!("Reached timeout ({timeout:?}) with no events, exiting ...");
                    exit(1);
                }
            }
        });
    }

    // Run device discovery
    let discover = adapter.discover_devices().await?;
    tokio::task::spawn(async move {
        pin_mut!(discover);
        while let Some(evt) = discover.next().await {
            log::info!("Discovery event: {:?}", evt);
        }
    });

    match &args.mode {
        Mode::Client {
            read,
            stream,
            turn_on,
            turn_off,
            report_interval,
            device,
        } => {
            let version = {
                let mut gatt = GattBoard::new(&device, adapter.clone());
                let version = gatt.version().await?;
                version
            };
            println!("Connected to board! Running version {}", version);
            let mut board = BurrBoard::new(&device, adapter.clone());
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
        }
        Mode::Gateway {
            http,
            user,
            password,
            enable_dfu,
            devices,
        } => {
            let dfu_client = Arc::new(FirmwareClient::new(
                http.clone(),
                user.clone(),
                password.clone(),
            ));
            let gateway = Arc::new(Gateway::new(http.clone(), user.clone(), password.clone()));
            let devices: Vec<String> = devices.split(",").map(|s| s.to_string()).collect();

            let mut tasks = Vec::new();
            for device in devices {
                let mut gatt = GattBoard::new(&device, adapter.clone());
                let version = gatt.version().await?;
                println!("Connected to {}! Running version {}", device, version);

                let mut board = BurrBoard::new(&device, adapter.clone());
                // Stream sensors
                let device_name = board.address().to_string();
                let last_event = last_event.clone();
                let gateway = gateway.clone();

                tasks.push(tokio::task::spawn(async move {
                    log::info!("Running data stream for '{device}'");
                    loop {
                        match Box::pin(board.stream_sensors()).await {
                            Ok(mut s) => {
                                let mut view = json!({});
                                loop {
                                    if let Some(n) = s.next().await {
                                        *last_event.lock().await = Instant::now();

                                        let previous = view.clone();
                                        merge(&mut view, &n);
                                        if previous != view {
                                            let payload = json! {
                                                {
                                                    "features": view,
                                                }
                                            };
                                            log::info!("Payload: {payload}");
                                            gateway.publish(&device_name, &payload).await;
                                        }
                                    } else {
                                        log::info!("Stream closed");
                                        sleep(Duration::from_secs(2)).await;
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!(
                                    "Error streaming sensor data from {}: {:?}",
                                    device_name,
                                    e
                                );
                                sleep(Duration::from_secs(2)).await;
                            }
                        }
                    }
                }));

                // Starting updater process
                if *enable_dfu {
                    let client = dfu_client.clone();
                    tasks.push(tokio::task::spawn(async move {
                        loop {
                            client.run(&mut gatt).await;
                            sleep(Duration::from_secs(10)).await;
                        }
                    }));
                }
            }
            for t in tasks {
                t.await?;
            }
        }
    }

    log::info!("BLE sensor disconnected, shutting down");
    Ok(())
}
