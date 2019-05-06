#[derive(Debug)]
pub enum Error {
    InvalidFps(Vec<f64>),
    InvalidResolution(Vec<(u32, u32)>),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}
