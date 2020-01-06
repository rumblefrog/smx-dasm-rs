 use std::fmt::{Display, Formatter};

use std::error::Error as StdError;
use std::io::Error as IoError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(IoError),

    InvalidMagic,
    InvalidSize,
    InvalidOffset,
    InvalidIndex,
    OffsetOverflow,
    SizeOverflow,

    Other(&'static str),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Io(ref inner) => inner.fmt(f),
            Error::InvalidMagic => write!(f, "Invalid magic header"),
            Error::InvalidSize => write!(f, "Invalid size"),
            Error::InvalidOffset => write!(f, "Invalid offset"),
            Error::InvalidIndex => write!(f, "Invalid index"),
            Error::OffsetOverflow => write!(f, "Offset overflow"),
            Error::SizeOverflow => write!(f, "Size overflow"),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref inner) => inner.description(),
            Error::InvalidMagic => "Invalid magic header",
            Error::InvalidSize => "Invalid size",
            Error::InvalidOffset => "Invalid offset",
            Error::InvalidIndex => "Invalid index",
            Error::OffsetOverflow => "Offset overflow",
            Error::SizeOverflow => "Size overflow",
            Error::Other(msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            Error::Io(ref inner) => Some(inner),
            _ => None,
        }
    }
}