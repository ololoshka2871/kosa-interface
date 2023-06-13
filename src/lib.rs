use std::{fmt::Display, io};
use tokio_util::codec::{Decoder, Encoder};

use bytes::{BufMut, BytesMut};

pub struct LineCodec;

pub enum Request {
    FF10,
    Start,
}

#[allow(non_snake_case)]
pub struct DataResponce {
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

pub enum Error {
    InvalidResponce,
    EmptyResponce,
}

impl DataResponce {
    fn from_raw(data: &[u8]) -> Result<Self, Error> {
        const DATAT_OFFSET: usize = 5;

        if data.len() >= DATAT_OFFSET + std::mem::size_of::<DataResponce>() {
            let mut res = [0u8; std::mem::size_of::<DataResponce>()];
            res.copy_from_slice(
                &data[DATAT_OFFSET..DATAT_OFFSET + std::mem::size_of::<DataResponce>()],
            );

            // if all data bytes are zero, then it's not a valid data
            if res.iter().all(|&x| x == 0x00) {
                return Err(Error::EmptyResponce);
            }

            let transformed: Vec<u8> = res
                .chunks(std::mem::size_of::<f32>())
                .map(|d| [d[1], d[0], d[3], d[2]])
                .flatten()
                .collect();

            res.copy_from_slice(&transformed);
            Ok(unsafe { std::mem::transmute_copy(&res) })
        } else {
            Err(Error::InvalidResponce)
        }
    }

    pub fn freq(&self) -> f32 {
        self._signal
    }
}

impl Display for DataResponce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "_T_P: {}\t_error: {}\t_F: {}\t_alpha: {}\t_signal: {}\t_noise: {}",
            self._T_P,
            self._error,
            self._F,
            self._alpha,
            self._signal,
            self._noise
        )
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
