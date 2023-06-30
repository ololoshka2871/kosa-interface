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
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> tokio_serial::Result<()> {
    let args = Cli::parse();

    let mut kosa = Kosa::new(args.port);

    loop {
        match kosa.get_measurement(Duration::from_millis(50)).await {
            Ok(res) => println!("Result: {}", res),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
