use futures::{stream::StreamExt, SinkExt};
use std::{io, time::Duration};
use tokio::time::sleep;
use tokio_util::codec::{Decoder, Encoder};

use bytes::{BufMut, BytesMut};
use tokio_serial::SerialPortBuilderExt;

struct LineCodec;

enum Request {
    FF10,
    Start,
}

#[allow(non_snake_case)]
struct DataResponce {
    // условные названия, так как они написаны в Q2view
    pub _T_P: f32,
    pub _error: f32,
    pub _F: f32,
    pub _alpha: f32,
    pub _a: f32,
    pub _phi: f32,
    pub _signal: f32,
    pub _noise: f32,
}

impl DataResponce {
    fn from_raw(data: &[u8]) -> Result<Self, ()> {
        const DATAT_OFFSET: usize = 5;

        if data.len() >= DATAT_OFFSET + std::mem::size_of::<DataResponce>() {
            let mut res = [0u8; std::mem::size_of::<DataResponce>()];
            res.copy_from_slice(
                &data[DATAT_OFFSET..DATAT_OFFSET + std::mem::size_of::<DataResponce>()],
            );

            let transformed: Vec<u8> = res
                .chunks(std::mem::size_of::<f32>())
                .map(|d| {
                    let r = [d[1], d[0], d[3], d[2]];
                    // println!("{:?} -> {:?}", d, r);
                    r
                })
                .flatten()
                .collect();

            res.copy_from_slice(&transformed);
            Ok(unsafe { std::mem::transmute_copy(&res) })
        } else {
            Err(())
        }
    }

    fn freq(&self) -> f32 {
        self._signal
    }
}

impl Decoder for LineCodec {
    type Item = DataResponce;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match DataResponce::from_raw(&src) {
            Ok(res) => {
                src.clear();
                Ok(Some(res))
            }
            Err(_) => Ok(None),
        }
    }

    fn framed<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Sized>(
        self,
        io: T,
    ) -> tokio_util::codec::Framed<T, Self>
    where
        Self: Sized,
    {
        tokio_util::codec::Framed::new(io, self)
    }
}

fn calc_crc(data: &[u8]) -> u16 {
    let mut crc = 0xFFFF;
    for x in data {
        crc ^= u16::from(*x);
        for _ in 0..8 {
            let crc_odd = (crc & 0x0001) != 0;
            crc >>= 1;
            if crc_odd {
                crc ^= 0xA001;
            }
        }
    }
    crc << 8 | crc >> 8
}

impl Encoder<Request> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, req_type: Request, buf: &mut BytesMut) -> Result<(), Self::Error> {
        const DEVICE_ADDRESS: u16 = 0xf801;
        const READ_HOLDINGS: u8 = 0x03;

        match req_type {
            Request::FF10 => {
                buf.reserve(
                    2 // addr
                    + 1 // cmd
                    + 2 //start adddr
                    + 2 // cells count to read
                    + 2, // crc
                );
                buf.put_u16(DEVICE_ADDRESS);
                buf.put_u8(READ_HOLDINGS);
                buf.put_u16(0xFF10);
                buf.put_u16(0x0100);
            }
            Request::Start => {
                buf.reserve(
                    1 // broadcast
                    + 1 // cmd
                    + 5 // data
                    + 2, // crc
                );
                buf.put_u8(0);
                buf.put_u8(0x64);
                buf.put_slice(&[0x03, 0x03, 0, 0, 0xff]);
            }
        }

        let crc = calc_crc(buf);
        buf.put_u16(crc);
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> tokio_serial::Result<()> {
    let port = tokio_serial::new("/dev/ttyUSB0", 1500000).open_native_async()?;
    let mut io = LineCodec.framed(port);

    loop {
        io.send(Request::Start).await?;
        sleep(Duration::from_millis(1000)).await;
        io.send(Request::FF10).await?;
        sleep(Duration::from_millis(100)).await;

        let res = io.next().await;
        let res = res.unwrap()?;
        println!("F={}", res.freq());
    }

    //Ok(())
}
