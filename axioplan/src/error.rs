use std::{io, num::ParseIntError, string::FromUtf8Error};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScopeError {
    #[error("received response was empty")]
    EmptyResponse,

    #[error("query validation failed; {0} != {1}")]
    QueryValidation(String, String),

    #[error("the response receieved was invalid")]
    InvalidResponse,

    #[error("the response contained an unparseable number: {0}")]
    InvalidNumber(#[from] ParseIntError),

    #[error("the response was not valid UTF-8")]
    InvalidUTF8(#[from] FromUtf8Error),

    #[error("communication with the device failed")]
    CommunicationFailure(#[from] io::Error),

    #[error("internal serial error")]
    SerialError(#[from] serialport::Error),

    #[error("value provided was out of the valid range; {0} > {1}")]
    OutOfRange(u64, u64),
}
