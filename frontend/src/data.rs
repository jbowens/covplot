use std::fmt;
use std::collections::HashMap;

pub struct DataSet {
    pub dates : Vec<chrono::NaiveDate>,
    pub series : Vec<Series>,
    pub regions : Vec<(String,Vec<Region>)>,
}

impl DataSet {
    pub fn new(dates : Vec<chrono::NaiveDate>, raw_series : Vec<Series>) -> DataSet {
        // Remove all series for minor localities.
        let mut series : Vec<Series> = raw_series
            .into_iter()
            .filter(|s| !s.region.is_minor_locality())
            .collect();

        // Construct a map from country to regions.
        let mut countries_map : HashMap<String, Vec<Region>> = HashMap::new();
        let mut countries_series : HashMap<String, Series> = HashMap::new();
        for s in series.iter() {
            match countries_map.get_mut(&s.region.country) {
                None => {
                    countries_map.insert(s.region.country.clone(), vec![s.region.clone()]);
                },
                Some(states) => {
                    states.push(s.region.clone());
                },
            }

            if !s.region.state.is_empty() {
                match countries_series.get_mut(&s.region.country) {
                    None => {
                        let mut aggregate_series = s.clone();
                        aggregate_series.region.state = "".to_string();
                        countries_series.insert(s.region.country.clone(), aggregate_series);
                    },
                    Some(existing_series) => {
                        let pts = existing_series.points.as_mut_slice();
                        for i in 0..pts.len() {
                            pts[i] = pts[i] + s.points[i];
                        }
                    },
                }
            }
        }
        let mut countries_agg = countries_series.into_iter().map(|(_, series)| series).collect();
        series.append(&mut countries_agg);

        let mut regions : Vec<(String, Vec<Region>)> = countries_map
            .into_iter()
            .map(|(country, mut regions)| {
                // Sort the states/provinces within this country by their names.
                regions.sort_by(|a, b| a.state.cmp(&b.state));
                (country, regions)
            })
            .collect();

        // Sort the countries by their names.
        regions.sort_by(|(a, _), (b, _)| a.cmp(&b));

        DataSet{dates, series, regions}
    }

    pub fn select(&self, regions : &[Region]) -> Vec<&Series> {
       self.series
           .iter()
           .filter(|s| regions.contains(&s.region))
           .collect()
    }
}

#[derive(Clone)]
pub struct Series {
    pub region : Region,
    pub data_type: DataType,
    pub series_type : SeriesType,
    pub points : Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Region {
    pub country : String,
    pub state : String,
}

impl Region {
    pub fn new(country : &str, state : &str) -> Region {
        Region{country: country.to_string(), state: state.to_string()}
    }

    pub fn is_minor_locality(&self) -> bool {
        // The data source contains some cities and counties
        // where data is more granular. These always have a
        // comma separting the minor locality from the state
        // or province.
        self.state.contains(",")
    }
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

#[derive(Clone, Copy)]
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
