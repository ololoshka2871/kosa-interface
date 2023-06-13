use std::time::Duration;

use futures::{SinkExt, StreamExt};

use tokio::time::sleep;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

use kosa_interface::{LineCodec, Request};

#[tokio::main(flavor = "current_thread")]
async fn main() -> tokio_serial::Result<()> {
    let port = tokio_serial::new("COM5", 1500000).open_native_async()?;
    let mut io = LineCodec.framed(port);

    loop {
        io.send(Request::Start).await?;
        sleep(Duration::from_millis(300)).await;
        io.send(Request::FF10).await?;
        sleep(Duration::from_millis(20)).await;

        let res = io.next().await;
        let res = res.unwrap()?;
        println!("{}", res);
    }

    //Ok(())
}
