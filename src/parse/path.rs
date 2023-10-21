use crate::error::InvalidUrl;

const NONE: u16 = u16::MAX;

pub struct PathAndQuery<'a> {
    data: &'a str,
    query: u16,
}

impl<'a> PathAndQuery<'a> {
    pub fn parse(data: &'a str) -> Result<Self, InvalidUrl> {
        let mut query = None;
        let mut fragment = None;

        if data.len() >= u16::MAX as usize {
            return Err(InvalidUrl::TooLong);
        }

        let mut iter = data.as_bytes().iter().enumerate();

        for (i, c) in &mut iter {
            match c {
                b'?' => {
                    query = Some(i as u16);
                    break;
                }
                b'#' => {
                    fragment = Some(i as u16);
                    break;
                }
                0x21
                | 0x24..=0x3B
                | 0x3D
                | 0x40..=0x5F
                | 0x61..=0x7A
                | 0x7C
                | 0x7E
                // Not allowed by spec, but used in the wild
                | b'"'
                | b'{'
                | b'}' => {}
                _ => return Err(InvalidUrl::InvalidUrlCodePoint),
            }
        }

        if query.is_some() {
            for (i, c) in iter {
                match c {
                    b'#' => {
                        fragment = Some(i as u16);
                        break;
                    }
                    0x21 | 0x24..=0x3B | 0x3D | 0x3F..=0x7E => {}
                    _ => return Err(InvalidUrl::InvalidUrlCodePoint),
                }
            }
        }

        let data = &data[..fragment.map_or(data.len(), |f| f as usize)];

        Ok(Self {
            data,
            query: query.unwrap_or(NONE),
        })
    }

    pub fn path(&self) -> &'a str {
        let ret = if self.query == NONE {
            self.data
        } else {
            &self.data[..self.query as usize]
        };

        if ret.is_empty() {
            return "/";
        }

        ret
    }

    pub fn query(&self) -> Option<&'a str> {
        if self.query == NONE {
            None
        } else {
            Some(&self.data[self.query as usize + 1..])
        }
    }
}
