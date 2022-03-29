use bluer::AdapterEvent;
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

    #[clap(short, long)]
    devices: String,

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

        #[clap(long)]
        enable_dfu: bool,
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

    let devices: Vec<&str> = args.devices.split(",").collect();
    let session = bluer::Session::new().await?;
    let adapter = Arc::new(session.default_adapter().await?);
    adapter.set_powered(true).await?;
    let discover = adapter.discover_devices().await?;
    pin_mut!(discover);

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

    match &args.mode {
        Mode::Client {
            read,
            stream,
            turn_on,
            turn_off,
            report_interval,
        } => {
            while let Some(evt) = discover.next().await {
                match evt {
                    AdapterEvent::DeviceAdded(a) if devices.contains(&a.to_string().as_str()) => {
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
                                    Err(err) => return Err(err.into()),
                                }
                            }
                            println!("Connected");
                        } else {
                            println!("Already connected");
                        }
                        let mut gatt =
                            GattBoard::new(&device.address().to_string(), adapter.clone());
                        let version = gatt.version().await?;
                        println!("Connected to board! Running version {}", version);
                        let board = BurrBoard::new(device);
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
                    AdapterEvent::DeviceRemoved(a) if devices.contains(&a.to_string().as_str()) => {
                        log::info!("Device removed: {}", a);
                    }
                    _ => {}
                }
            }
        }

        Mode::Gateway {
            http,
            user,
            password,
            enable_dfu,
        } => {
            let dfu_client = Arc::new(FirmwareClient::new(
                http.clone(),
                user.clone(),
                password.clone(),
            ));
            let gateway = Arc::new(Gateway::new(http.clone(), user.clone(), password.clone()));

            while let Some(evt) = discover.next().await {
                match evt {
                    AdapterEvent::DeviceAdded(a) if devices.contains(&a.to_string().as_str()) => {
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
                                    Err(err) => return Err(err.into()),
                                }
                            }
                            println!("Connected");
                        } else {
                            println!("Already connected");
                        }

                        let mut gatt =
                            GattBoard::new(&device.address().to_string(), adapter.clone());
                        let version = gatt.version().await?;
                        println!(
                            "Connected to {}! Running version {}",
                            device.address(),
                            version
                        );

                        let board = Arc::new(BurrBoard::new(device));
                        // Stream sensors
                        let device_name = board.address().to_string();
                        let last_event = last_event.clone();
                        let gateway = gateway.clone();

                        tokio::task::spawn(async move {
                            log::info!("Running data stream for '{a}'");

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
                                                log::debug!("Payload: {payload}");
                                                match serde_json::to_vec(&payload) {
                                                    Ok(payload) => {
                                                        gateway
                                                            .publish(&device_name, &payload[..])
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        log::warn!(
                                                            "Error encoding payload: {:?}",
                                                            e
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::warn!(
                                        "Error streaming sensor data from {}: {:?}",
                                        device_name,
                                        e
                                    );
                                }
                            }
                        });

                        // Starting updater process
                        if *enable_dfu {
                            let client = dfu_client.clone();
                            tokio::task::spawn(async move {
                                loop {
                                    client.run(&mut gatt).await;
                                    sleep(Duration::from_secs(10)).await;
                                }
                            });
                        }
                    }
                    AdapterEvent::DeviceRemoved(a) if devices.contains(&a.to_string().as_str()) => {
                        log::info!("Device removed: {}", a);
                    }
                    _ => {}
                }
            }
        }
    }

    log::info!("BLE sensor disconnected, shutting down");
    Ok(())
}
