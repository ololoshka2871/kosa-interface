use bytes::{BufMut, BytesMut};

use tokio_util::codec::{Decoder, Encoder};

use crate::protocol::{DataResponce, Request};

pub(crate) struct LineCodec;

impl Decoder for LineCodec {
    type Item = DataResponce;
    type Error = crate::protocol::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match DataResponce::from_raw(&src) {
            Err(crate::protocol::Error::EmptyResponce) => Ok(None),
            Ok(res) => {
                src.clear();
                Ok(Some(res))
            }
            Err(crate::protocol::Error::ZeroResponce) => {
                src.clear();
                Ok(None)
            }
            Err(e) => {
                src.clear();
                Err(e)
            }
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

impl Encoder<Request> for LineCodec {
    type Error = crate::protocol::Error;

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
