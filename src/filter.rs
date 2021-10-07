use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
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
mod test {
    use super::*;

    #[test]
    fn filter_parse_exact_ok() -> Result<(), ParseError> {
        let filter = Filter::from_str("1")?;
        let expected = Filter {
            start: 1,
            end: None,
        };

        assert_eq!(filter, expected);

        Ok(())
    }

    #[test]
    fn filter_parse_range_ok() -> Result<(), ParseError> {
        let filter = Filter::from_str("2:4")?;
        let expected = Filter {
            start: 2,
            end: Some(4),
        };

        assert_eq!(filter, expected);
        Ok(())
    }

    #[test]
    fn filter_parse_range_start_ok() -> Result<(), ParseError> {
        let filter = Filter::from_str("2:")?;
        let expected = Filter {
            start: 2,
            end: Some(0),
        };

        assert_eq!(filter, expected);
        Ok(())
    }

    #[test]
    fn filter_parse_range_end_ok() -> Result<(), ParseError> {
        let filter = Filter::from_str(":4")?;
        let expected = Filter {
            start: 1,
            end: Some(4),
        };

        assert_eq!(filter, expected);
        Ok(())
    }

    #[test]
    fn filter_parse_range_full_ok() -> Result<(), ParseError> {
        let filter = Filter::from_str(":")?;
        let expected = Filter {
            start: 1,
            end: Some(0),
        };

        assert_eq!(filter, expected);
        Ok(())
    }

    #[test]
    fn filter_parse_negative_err() -> Result<(), ParseError> {
        let filter = Filter::from_str("1:-1");

        assert!(matches!(filter, Err(ParseError::ParseIntFailed(_))));

        Ok(())
    }

    #[test]
    fn filter_parse_non_numeric_err() -> Result<(), ParseError> {
        let filter = Filter::from_str("non:numeric");

        assert!(matches!(filter, Err(ParseError::ParseIntFailed(_))));
        Ok(())
    }

    #[test]
    fn filter_parse_range_invalid_err() -> Result<(), ParseError> {
        let filter = Filter::from_str("4:2");

        assert!(matches!(filter, Err(ParseError::InvalidFilter { .. })));
        Ok(())
    }

    #[test]
    fn filterset_apply_exact_true() -> Result<(), ParseError> {
        let filters = vec![Filter {
            start: 2,
            end: None,
        }];
        let index = 2;

        assert_eq!(FilterSet::new(filters).apply(index), true);
        Ok(())
    }

    #[test]
    fn filterset_apply_exact_false() -> Result<(), ParseError> {
        let filters = vec![Filter {
            start: 2,
            end: None,
        }];
        let index = 4;

        assert_eq!(FilterSet::new(filters).apply(index), false);
        Ok(())
    }

    #[test]
    fn filterset_apply_range_full_true() -> Result<(), ParseError> {
        let filters = vec![
            Filter {
                start: 1,
                end: Some(0),
            },
            Filter {
                start: 2,
                end: None,
            },
        ];
        let index = 4;

        assert_eq!(FilterSet::new(filters).apply(index), true);
        Ok(())
    }

    #[test]
    fn filterset_apply_range_inside_true() -> Result<(), ParseError> {
        let filters = vec![
            Filter {
                start: 1,
                end: Some(4),
            },
            Filter {
                start: 2,
                end: None,
            },
        ];
        let index = 3;

        assert_eq!(FilterSet::new(filters).apply(index), true);
        Ok(())
    }

    #[test]
    fn filterset_apply_range_inclusive_true() -> Result<(), ParseError> {
        let filters = vec![
            Filter {
                start: 1,
                end: Some(4),
            },
            Filter {
                start: 2,
                end: None,
            },
        ];
        let index = 4;

        assert_eq!(FilterSet::new(filters).apply(index), true);
        Ok(())
    }

    #[test]
    fn filterset_apply_range_before_false() -> Result<(), ParseError> {
        let filters = vec![
            Filter {
                start: 3,
                end: Some(5),
            },
            Filter {
                start: 2,
                end: Some(4),
            },
        ];
        let index = 1;

        assert_eq!(FilterSet::new(filters).apply(index), false);
        Ok(())
    }

    #[test]
    fn filterset_apply_range_after_false() -> Result<(), ParseError> {
        let filters = vec![
            Filter {
                start: 3,
                end: Some(5),
            },
            Filter {
                start: 2,
                end: Some(4),
            },
        ];
        let index = 6;

        assert_eq!(FilterSet::new(filters).apply(index), false);
        Ok(())
    }

    #[test]
    fn filterset_apply_range_between_false() -> Result<(), ParseError> {
        let filters = vec![
            Filter {
                start: 4,
                end: Some(5),
            },
            Filter {
                start: 1,
                end: Some(2),
            },
        ];
        let index = 3;

        assert_eq!(FilterSet::new(filters).apply(index), false);
        Ok(())
    }
}
