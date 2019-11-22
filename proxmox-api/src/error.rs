use std::fmt;

use failure::Fail;

#[doc(hidden)]
pub use hyper::StatusCode;

#[derive(Debug, Fail)]
pub struct HttpError {
    pub code: StatusCode,
    pub message: String,
}

impl HttpError {
    pub fn new(code: StatusCode, message: String) -> Self {
        HttpError { code, message }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[macro_export]
macro_rules! http_err {
    ($status:ident, $msg:expr) => {{
        ::failure::Error::from($crate::error::HttpError::new(
            $crate::error::StatusCode::$status,
            $msg,
        ))
    }};
}