use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ComicError {
    IoError(io::Error),
    InvalidPath,
    ComicNotFound,
    NoCoverFound,
    ZipError(zip::result::ZipError),
}

impl fmt::Display for ComicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComicError::IoError(e) => write!(f, "IO error: {}", e),
            ComicError::InvalidPath => write!(f, "Invalid path"),
            ComicError::ComicNotFound => write!(f, "Comic not found"),
            ComicError::NoCoverFound => write!(f, "No cover found in comic"),
            ComicError::ZipError(e) => write!(f, "Zip error: {}", e),
        }
    }
}

impl std::error::Error for ComicError {}

impl From<io::Error> for ComicError {
    fn from(error: io::Error) -> Self {
        ComicError::IoError(error)
    }
}

impl From<zip::result::ZipError> for ComicError {
    fn from(error: zip::result::ZipError) -> Self {
        ComicError::ZipError(error)
    }
}

impl From<notify::Error> for ComicError {
    fn from(error: notify::Error) -> Self {
        ComicError::IoError(io::Error::new(io::ErrorKind::Other, error))
    }
}