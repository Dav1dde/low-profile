use crate::http::InvalidMethod;

#[derive(Debug)]
pub enum InvalidUrl {
    TooLong,
    InvalidUrlCodePoint,
}

#[derive(Debug)]
pub enum ProtocolError {
    InvalidUrl(InvalidUrl),
    InvalidMethod(InvalidMethod),
    Parser(httparse::Error),
}
