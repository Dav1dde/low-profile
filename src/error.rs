#[derive(Debug)]
pub enum InvalidUrl {
    TooLong,
    InvalidUrlCodePoint,
}
