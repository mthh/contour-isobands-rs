use std::error::Error as StdError;
use std::fmt;
use std::result;

/// A crate private constructor for `Error`.
pub(crate) fn new_error(kind: ErrorKind) -> Error {
    Error(Box::new(kind))
}

/// A type alias for `Result<T, csv::Error>`.
pub type Result<T> = result::Result<T, Error>;

/// An error that can occur when computing contours.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    /// Return the specific type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    /// Unwrap this error into its underlying type.
    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

/// The specific type of an error.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    BadDimension,
    JsonError(serde_json::error::Error),
    Unexpected,
    UnknownEnterType,
    UnexpectedCVAL,
    UnexpectedOutOfGridMove,
    OutOfBounds,
    BadIntervals,
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Error {
        new_error(ErrorKind::JsonError(err))
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self.0 {
            ErrorKind::JsonError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::JsonError(ref err) => err.fmt(f),
            ErrorKind::BadDimension => write!(
                f,
                "The length of provided values doesn't match the (dx, dy) dimensions of the grid"
            ),
            ErrorKind::Unexpected => write!(f, "Unexpected error while computing contours"),
            ErrorKind::UnexpectedCVAL => write!(f, "Unexpected cval"),
            ErrorKind::UnknownEnterType => write!(f, "Unknown enter type"),
            ErrorKind::OutOfBounds => write!(f, "Out of bounds"),
            ErrorKind::UnexpectedOutOfGridMove => write!(f, "Unexpected out of grid move"),
            ErrorKind::BadIntervals => write!(f, "Intervals argument must have at least 2 elements (representing the lowerbound and the upperbound of the band to compute)")
        }
    }
}
