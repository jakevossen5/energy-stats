use std::{println, todo};

use chrono::{Date, DateTime, FixedOffset, Local, NaiveDateTime, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::{API_KEY_KEY, SERIES_ID_KEY};
#[derive(Debug, Serialize, Deserialize)]

pub struct Series {
    series_id: String,
    name: String,
    units: String,
    f: String,
    description: String,
    // strart: Date<Utc>,
    // #[serde(skip)]
    // end: Date<Utc>,
    // #[serde(skip)]
    // updated: DateTime<Utc>,
    data: Vec<(String, f64)>,
}

impl Series {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub request: Request,
    pub series: Vec<Series>,
}

impl Response {
    pub fn new(series_id: &str, client: &Client) -> Self {
        println!("doing api request for {}", series_id);
        client
            .get("http://api.eia.gov/series/")
            .query(&[
                (
                    API_KEY_KEY,
                    crate::env_helper::get_api_key().unwrap().as_ref(),
                ),
                (SERIES_ID_KEY, series_id),
            ])
            .send()
            .unwrap()
            .json::<Response>()
            .unwrap()
    }
    pub fn get_dt_data(&self) -> Vec<(Option<NaiveDateTime>, f64)> {
        self.series[0]
            .data
            .iter()
            .map(|(date_str, val)| {
                // println!("Date_str: {}", date_str);
                let new_date: String = date_str
                    .chars()
                    .filter(|c| *c != 'Z')
                    .chain(":00:00".chars())
                    .collect::<String>();

                let date = NaiveDateTime::parse_from_str(&new_date, "%Y%m%dT%H:%M:%S").ok();
                (date, *val)
            })
            .collect()
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    command: String,
    series_id: String,
}
