use bluer::AdapterEvent;
use clap::Parser;
use futures::{pin_mut, StreamExt};
use log;
use serde_json::json;

mod board;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    device: String,

    #[clap(short, long)]
    wait: bool,
}

use crate::board::BurrBoard;

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
async fn main() -> bluer::Result<()> {
    let args = Args::parse();
    stderrlog::new().verbosity(0).init().unwrap();

    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    let discover = adapter.discover_devices().await?;
    pin_mut!(discover);

    while let Some(evt) = discover.next().await {
        match evt {
            AdapterEvent::DeviceAdded(addr) => {
                log::info!("Discovered {}", addr);
                if addr.to_string() == args.device {
                    let device = adapter.device(addr)?;
                    if !device.is_connected().await? {
                        let mut retries = 2;
                        loop {
                            match device.connect().await {
                                Ok(()) => break,
                                Err(_) if retries > 0 => {
                                    retries -= 1;
                                }
                                Err(err) => Err(err)?,
                            }
                        }
                    }
                    log::info!("Found our device!");
                    let board = BurrBoard::new(device);
                    if args.wait {
                        let s = board.stream_sensors().await?;
                        pin_mut!(s);
                        let mut view = json!({});
                        while let Some(n) = s.next().await {
                            merge(&mut view, &n);
                            println!("{}", view);
                        }
                    } else {
                        let sensor = board.read_sensors().await?;
                        println!("{}", sensor);
                        return Ok(());
                    }
                }
            }
            AdapterEvent::DeviceRemoved(addr) => {
                log::info!("Device removed {}", addr);
            }
            _ => {}
        }
    }

    log::info!("BLE sensor disconnected, shutting down");
    Ok(())
}
