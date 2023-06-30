//! Example for plotting the signal/noise ratio of the KOSA device.
//! use [this](https://github.com/ololoshka2871/livechart) tool to plot output data (JSON).

use std::time::Duration;

use clap::Parser;

use kosa_interface::Kosa;

/// Kosa test tool
#[derive(Parser)]
#[allow(non_snake_case)]
struct Cli {
    /// Serial port to use, e.g. COM1 or /dev/ttyUSB0
    #[clap(short, long)]
    port: String,

    // show "noise"
    #[clap(short, long)]
    noise: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> tokio_serial::Result<()> {
    let args = Cli::parse();

    let mut kosa = Kosa::new(args.port);
    let stdout = std::io::stdout();

    loop {
        match kosa.get_measurement(Duration::from_millis(50)).await {
            Ok(res) => {
                let output = if args.noise {
                    serde_json::json!(
                        {
                            "signal": res.freq(),
                            "noise": res._noise,
                        }
                    )
                } else {
                    serde_json::json!(
                        {
                            "signal": res.freq(),
                        }
                    )
                };
                serde_json::to_writer(&stdout, &output).unwrap();
                println!();
            }
            Err(_) => { /* skip */ }
        }
    }
}
