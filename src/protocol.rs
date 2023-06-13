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

impl DataResponce {
    pub fn from_raw(data: &[u8]) -> Result<Self, Error> {
        const DATAT_OFFSET: usize = 5;

        if data.len() >= DATAT_OFFSET + std::mem::size_of::<DataResponce>() {
            let mut res = [0u8; std::mem::size_of::<DataResponce>()];
            res.copy_from_slice(
                &data[DATAT_OFFSET..DATAT_OFFSET + std::mem::size_of::<DataResponce>()],
            );

            // if all data bytes are zero, then it's not a valid data
            if res.iter().all(|&x| x == 0x00) {
                return Err(Error::ZeroResponce);
            }

            let transformed: Vec<u8> = res
                .chunks(std::mem::size_of::<f32>())
                .map(|d| [d[1], d[0], d[3], d[2]])
                .flatten()
                .collect();

            res.copy_from_slice(&transformed);
            Ok(unsafe { std::mem::transmute_copy(&res) })
        } else {
            Err(Error::EmptyResponce)
        }
    }

    pub fn freq(&self) -> f32 {
        self._signal
    }
}

impl std::fmt::Display for DataResponce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "_F: {}\t_alpha: {}\t_signal: {}\t_noise: {}",
            self._F, self._alpha, self._signal, self._noise
        )
    }
}

#[derive(Debug)]
pub enum Error {
    Timeout,
    InvalidResponce,
    ZeroResponce,
    EmptyResponce,
    UnexpectedEndOfStream,
    Unknown(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Unknown(value)
    }
}
