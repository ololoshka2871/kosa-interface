mod line_codec;
mod protocol;

use std::time::Duration;

use futures::{SinkExt, StreamExt};

use tokio::time::sleep;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

use protocol::{Error, Request};

pub struct Kosa {
    io: tokio_util::codec::Framed<tokio_serial::SerialStream, line_codec::LineCodec>,
}

impl Kosa {
    pub fn new<'a>(port: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        let port = tokio_serial::new(port, 1500000)
            .open_native_async()
            .unwrap();
        Self {
            io: line_codec::LineCodec.framed(port),
        }
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        self.io.send(Request::Start).await
    }

    pub async fn read_responce(
        &mut self,
        timeout: Duration,
    ) -> Result<protocol::DataResponce, Error> {
        self.io.send(Request::FF10).await?;
        sleep(Duration::from_millis(20)).await; // just in case

        let res = tokio::time::timeout(timeout, self.io.next()).await;
        match res {
            Ok(Some(r)) => r,
            Ok(None) => Err(Error::UnexpectedEndOfStream),
            Err(_) => Err(Error::Timeout),
        }
    }

    pub async fn get_measurement(
        &mut self,
        timeout: Duration,
    ) -> Result<protocol::DataResponce, Error> {
        self.start().await?; // start measurement
        sleep(Duration::from_millis(300)).await; // awating for processing
        self.read_responce(timeout).await
    }
}
