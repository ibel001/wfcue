use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrackMSFParseError {
    #[error("Invalid timestamp: {0}")]
    InvalidTimeStamp(String),
}

#[derive(Debug, PartialEq)]
pub struct TrackMSF {
    minutes: u8,
    seconds: u8,
    fractions: u8,
}

#[allow(dead_code)]
impl TrackMSF {
    pub fn new(seconds: f64) -> Self {
        let (m, s) = Self::divmod(seconds, 60.0);
        let ms = s % 1.0;

        TrackMSF {
            minutes: m as u8,
            seconds: s as u8,
            fractions: (ms * 75.0).round() as u8,
        }
    }

    pub fn minutes(&self) -> u8 {
        self.minutes
    }

    pub fn seconds(&self) -> u8 {
        self.seconds
    }

    pub fn fractions(&self) -> u8 {
        self.fractions
    }

    pub fn to_duration_seconds(&self) -> f64 {
        ((self.minutes as u64 * 60) + self.seconds as u64) as f64
            + (self.fractions as f64 * 0.013333333333)
    }

    fn divmod(x: f64, y: f64) -> (f64, f64) {
        let quotient = x / y;
        let remainder = x % y;
        (quotient, remainder)
    }
}

impl TryFrom<&str> for TrackMSF {
    type Error = TrackMSFParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let err = || TrackMSFParseError::InvalidTimeStamp(value.to_owned());

        let split = value.split(':').collect::<Vec<_>>();
        if split.len() != 3 {
            Err(err())?
        }

        let numbers = split
            .into_iter()
            .map(|s| s.parse::<u8>().map_err(|_| err()))
            .collect::<Result<Vec<_>, _>>()?;
        if numbers.iter().any(|&n| n >= 100) {
            Err(err())?
        }

        Ok(Self {
            minutes: numbers[0],
            seconds: numbers[1],
            fractions: numbers[2],
        })
    }
}

impl fmt::Display for TrackMSF {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:0>2}:{:0>2}:{:0>2}",
            self.minutes, self.seconds, self.fractions
        )
    }
}
