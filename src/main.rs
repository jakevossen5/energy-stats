use std::cmp::{max, Ordering};

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use log::debug;
use reqwest::blocking::Client;
use serde::de::value;
use series::Response;
use source::Source;

mod env_helper;
mod series;
mod source;

const API_KEY_KEY: &'static str = "api_key";
const SERIES_ID_KEY: &'static str = "series_id";

struct Sources {
    pub sources: Vec<Source>,
}

impl Sources {
    pub fn new(sources: Vec<Source>) -> Self {
        let mut new = Self { sources };
        let total = new.total();
        for s in &mut new.sources {
            s.update_percent_of_total(total);
        }
        new.sources.sort_by(|a, b| {
            let a_v = a.percent_of_total().unwrap();
            let b_v = b.percent_of_total().unwrap();
            debug!("Trying to compare a_v: {} to b_v {}", a_v, b_v);
            a_v.partial_cmp(&b_v).unwrap_or(Ordering::Equal)
        });
        new.sources.reverse();
        new
    }
    pub fn total(&self) -> f64 {
        self.sources.iter().map(|s| s.get_last_data().1).sum()
    }
    fn average_over_n_hours(&self, hours: usize) -> f64 {
        self.total_over_n_hours(hours) / hours as f64
    }
    fn total_over_n_hours(&self, hours: usize) -> f64 {
        self.sources
            .iter()
            .map(|s| s.total_over_n_hours(hours))
            .sum()
    }
}

fn main() -> () {
    env_logger::init();
    // const PREFIX: &'static str =
    const HYDRO_SERIES_ID: (&'static str, &'static str, Option<&'static str>) =
        ("Hydro", "EBA.PSCO-ALL.NG.WAT.H", None);
    const GAS_SERIES_ID: (&'static str, &'static str, Option<&'static str>) = (
        "Natural Gas",
        "EBA.PSCO-ALL.NG.NG.H",
        Some("EMISS.CO2-C-NGEIB-CO.A"),
    );
    const OIL_SERIES_ID: (&'static str, &'static str, Option<&'static str>) = (
        "Oil",
        "EBA.PSCO-ALL.NG.OIL.H",
        Some("EMISS.CO2-C-PCEIB-CO.A"),
    );
    const SOLAR_SERIES_ID: (&'static str, &'static str, Option<&'static str>) =
        ("Solar", "EBA.PSCO-ALL.NG.SUN.H", None);
    const WIND_SERIES_ID: (&'static str, &'static str, Option<&'static str>) =
        ("Wind", "EBA.PSCO-ALL.NG.WND.H", None);
    const COAL_SERIES_ID: (&'static str, &'static str, Option<&'static str>) = (
        "Coal",
        "EBA.PSCO-ALL.NG.COL.H",
        Some("EMISS.CO2-C-PCEIB-CO.A"),
    );

    let series = vec![
        HYDRO_SERIES_ID,
        GAS_SERIES_ID,
        OIL_SERIES_ID,
        SOLAR_SERIES_ID,
        WIND_SERIES_ID,
        COAL_SERIES_ID,
    ];

    let client = reqwest::blocking::Client::new();

    let mut sources_vec = Vec::with_capacity(series.len());

    for (name, id, carbon_output) in series {
        sources_vec.push(Source::new(&client, name, id, carbon_output));
    }

    let sources = Sources::new(sources_vec);

    let total = sources.total();
    let average_over_past_month = sources.average_over_n_hours(24 * 31);
    // let current_co2 = sources.current_co2();
    println!("current total: {}", total);
    println!("average over past month: {:.2}", average_over_past_month);

    for source in sources.sources {
        let percent = source.percent_of_total().unwrap();
        let monthly_average = source.average_over_past_n_hours(24 * 31);
        let more_than_monthly_average = source.get_last_val() > monthly_average;
        println!(
            "{}:\n\tcurrent percent: {:.3}%\n\tcurrent actual: {:.2}\n\tmetric tons of carbon: {:.2}\n\tcurrent megawatts: {}\n\tmontly average: {:.2}\n\tmore than montly average: {}",
            source.name(),
            percent,
            source.get_last_val(),
            source.carbon_equiv(),
            source.get_last_val(),
            monthly_average,
            more_than_monthly_average
        );
    }

    println!("finished!");
}
