use std::time::Duration;

use futures::{SinkExt, StreamExt};

use tokio::time::sleep;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

use kosa_interface::{LineCodec, Request};

#[tokio::main(flavor = "current_thread")]
async fn main() -> tokio_serial::Result<()> {
    let port = tokio_serial::new("COM5", 1500000)
        //.timeout(Duration::from_millis(100))
        .open_native_async()?;
    let mut io = LineCodec.framed(port);

    loop {
        println!("Sending start request");
        io.send(Request::Start).await?;
        sleep(Duration::from_millis(300)).await; // awating for processing
        println!("Sending get data request");
        io.send(Request::FF10).await?;
        sleep(Duration::from_millis(20)).await;

        println!("Awaiting response");

        let res =  tokio::time::timeout(Duration::from_millis(50), io.next()).await;
        match res {
            Ok(Some(Ok(data))) => println!("{}", data),
            Ok(Some(Err(e))) => println!("Error: {}", e),
            Ok(None) => break,
            Err(_) => println!("Timeout"),
        }
    }

    Ok(())
}
