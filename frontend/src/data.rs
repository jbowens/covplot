use std::fmt;
use std::collections::HashMap;
use std::hash::Hasher;
use plotters::prelude::*;
use std::collections::hash_map::DefaultHasher;

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
            //.filter(|s| !s.region.is_minor_locality()) // Is this necessary???
            .collect();

        // Construct a map from country to regions.
        let mut countries_series : HashMap<String, Series> = HashMap::new();
        for s in series.iter() {
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

        // TODO: temporarily we're removing all series that aren't at the country level
        // , and that don't have at least 100 cases.
        let series : Vec<Series> = series
            .into_iter()
            .filter(|s| s.region.state.is_empty())
            .filter(|s| s.points.last().map(|x| *x).unwrap_or(0.0) >= 100.0)
            .collect();

        let mut countries_map : HashMap<String, Vec<Region>> = HashMap::new();
        for s in series.iter() {
            match countries_map.get_mut(&s.region.country) {
                None => {
                    countries_map.insert(s.region.country.clone(), vec![s.region.clone()]);
                },
                Some(states) => {
                    states.push(s.region.clone());
                },
            }
        }

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

    pub fn color(&self) -> impl Color {
        // For some common countries, bake a specific color value.
        // For all the other countries, compute a specific color by
        // hashing the region strings and mapping it into a reasonable,
        // legible portion of the HSL color space.
        match (self.country.as_str(), self.state.as_str()) {
            ("US", "") => HSLColor(0.60277, 1.0, 0.59),
            ("China", "") => HSLColor(0.01944, 0.87, 0.47),
            ("Italy", "") => HSLColor(0.4166, 1.0, 0.45),
            ("Spain", "") => HSLColor(0.116, 0.99, 0.58),
            _ => {
                let mut hasher = DefaultHasher::new();
                hasher.write(self.country.as_bytes());
                hasher.write(self.state.as_bytes());
                let idx = hasher.finish();

                let max_u64 = u64::max_value() as f64;
                let pct = idx as f64 / max_u64; // 0 ≤ pct ≤ 1
                let s = (idx % 113) as f64 / 113.0; // 0 ≤ s ≤ 1
                let l = (idx % 3463) as f64 / 3463.0; // 0 ≤ l ≤ 1
                HSLColor(pct, 0.25 + (s/2.0), 0.4 + (l/4.0))
            },
        }
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
