use std::fmt;
use chrono::prelude::*;

pub struct Series {
    pub region : Region,
    pub data_type: DataType,
    pub series_type : SeriesType,
    pub points : Vec<Point>,
}

pub struct Point {
    pub date : NaiveDate,
    pub value : u64,
}

pub struct Region {
    pub country : String,
    pub state : String,
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.state.is_empty() {
            write!(f, "{}", &self.country)
        } else {
            write!(f, "{}", &self.state)
        }
    }
}

pub enum SeriesType {
    Change,
    Total,
}

impl fmt::Display for SeriesType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           SeriesType::Change => write!(f, "Change"),
           SeriesType::Total => write!(f, "Total"),
       }
    }
}

#[derive(Clone, Copy)]
pub enum DataType {
    Confirmed,
    Recovered,
    Deaths,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           DataType::Confirmed => write!(f, "Confirmed"),
           DataType::Recovered => write!(f, "Recovered"),
           DataType::Deaths => write!(f, "Deaths"),
       }
    }
}
