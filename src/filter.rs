use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug)]
pub struct Filter {
    start: u32,
    end: Option<u32>,
}

impl FromStr for Filter {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let filter: Vec<&str> = s.split(':').collect();

        let start = match filter.get(0) {
            Some(&"") => 1,
            Some(&n) => n.parse()?,
            None => 1,
        };

        let end = match filter.get(1) {
            Some(&"") => Some(0),
            Some(&n) => {
                let end = n.parse()?;
                if end < start {
                    return Err(ParseError::InvalidFilter {
                        reason: format!("end [{}] cannot be before start [{}]", end, start),
                    });
                }
                Some(end)
            }
            None => None,
        };

        Ok(Filter { start, end })
    }
}

#[derive(Debug)]
pub enum ParseError {
    ParseIntFailed(ParseIntError),
    InvalidFilter { reason: String },
}

impl From<ParseIntError> for ParseError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseIntFailed(error)
    }
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ParseIntFailed(ref err) => write!(f, "failed to parse filter: {}", err),
            Self::InvalidFilter { ref reason } => write!(f, "invalid filter: {}", reason),
        }
    }
}

pub struct FilterSet(Vec<Filter>);

impl FilterSet {
    pub fn new(filters: Vec<Filter>) -> Self {
        Self(filters)
    }

    pub fn apply(&self, index: u32) -> bool {
        for filter in self.0.iter() {
            if index < filter.start {
                continue;
            }

            if match filter.end {
                Some(0) => true,
                Some(n) => index <= n,
                None => index == filter.start,
            } {
                return true;
            }
        }

        false
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod test {}
