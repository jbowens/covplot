use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use csv;
use crate::data::*;
use chrono::prelude::*;
use std::num::ParseIntError;

static CSSE_TIME_SERIES_CONFIRMED : &str = "https://raw.githubusercontent.com/CSSEGISandData/COVID-19/master/csse_covid_19_data/csse_covid_19_time_series/time_series_covid19_confirmed_global.csv";

// TODO: fixup errors

pub async fn query() -> Result<DataSet, String> {
    let data = download_csv(CSSE_TIME_SERIES_CONFIRMED)
        .await
        .map_err(|e| format!("error downloading csv: {:?}", e))?;

    parse_csv(DataType::Confirmed, data.as_bytes())
        .map_err(|e| format!("unable to parse CSSE csv data: {:?}", e))
}

fn parse_csv(typ : DataType, raw_data : &[u8]) -> Result<DataSet, csv::Error> {
    let mut rdr = csv::Reader::from_reader(raw_data);
    let headers = rdr.headers()?;

    let dates_result : Result<Vec<NaiveDate>, chrono::format::ParseError> = headers
        .iter()
        .skip(4)
        .map(|x| NaiveDate::parse_from_str(x, "%m/%d/%Y"))
        .collect();

    let dates = dates_result.unwrap(); // TODO: don't unwrap

    let mut series = vec![];
    for result in rdr.records() {
        let record = result?;
        let region = Region{
            country: record.get(1).unwrap_or("").to_string(),
            state: record.get(0).unwrap_or("").to_string(),
        };

        // TODO: if a data point is missing (eg, it's the empty string),
        // don't replace it with 0.
        let points: Result<Vec<f64>, ParseIntError> = record
            .iter()
            .skip(4)
            .map(|x| if x == "" { Ok(0) } else { u64::from_str_radix(x, 10) })
            .map(|x| x.map(|y| y as f64))
            .collect();

        series.push(Series{
            region: region,
            data_type: typ,
            series_type: SeriesType::Change,
            points: points.unwrap(), // TODO: don't unwrap
        });
    }

    Ok(DataSet::new(dates, series))
}

async fn download_csv(url : &str) -> Result<String, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts).unwrap();
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`,
    // await it and convert the resulting `JsValue` to a String.
    Ok(JsFuture::from(resp.text()?).await?.as_string().unwrap())
}
