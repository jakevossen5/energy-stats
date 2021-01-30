use chrono::NaiveDateTime;
use reqwest::blocking::Client;

use crate::{series::Response, API_KEY_KEY, SERIES_ID_KEY};

pub struct Source {
    name: &'static str,
    series_id: &'static str,
    api_response: Response,
    percent_of_total: Option<f64>,
    carbon_equiv_metric_tons: f64,
}

impl Source {
    pub fn new(
        client: &Client,
        name: &'static str,
        series_id: &'static str,
        carbon_coeff_series_id: Option<&'static str>,
    ) -> Self {
        let main_data = Response::new(series_id, client);

        let mut source = Self {
            name,
            series_id,
            api_response: main_data,
            percent_of_total: None,
            carbon_equiv_metric_tons: 0.0,
        };

        if let Some(id) = carbon_coeff_series_id {
            let carbon_coeff_data = Response::new(id, client);
            let last_carbon_coeff = carbon_coeff_data.get_dt_data().first().unwrap().1;
            let last_megawhatt_hours = source.get_last_val();
            let last_val_btu = last_megawhatt_hours * 3_412_141.63;
            let last_val_mil_btu = last_val_btu / 1_000_000.0;
            source.carbon_equiv_metric_tons = last_val_mil_btu * last_carbon_coeff / 1000.0;
        }

        source
    }
    pub fn get_last_data(&self) -> (Option<NaiveDateTime>, f64) {
        *self.api_response.get_dt_data().first().unwrap()
    }
    pub fn get_last_val(&self) -> f64 {
        self.get_last_data().1
    }

    pub fn update_percent_of_total(&mut self, total: f64) {
        let last_val = if self.get_last_val() < 0.0 {
            0.0
        } else {
            self.get_last_val()
        };
        self.percent_of_total = Some((last_val / total) * 100.0);
    }
    pub fn percent_of_total(&self) -> Option<f64> {
        self.percent_of_total
    }

    pub fn average_over_past_n_hours(&self, hours: usize) -> f64 {
        self.total_over_n_hours(hours) / hours as f64
    }

    pub fn total_over_n_hours(&self, hours: usize) -> f64 {
        self.api_response
            .get_dt_data()
            .iter()
            .take(hours)
            .map(|(_, val)| zero_or_more(val))
            .sum::<f64>()
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
    pub fn carbon_equiv(&self) -> f64 {
        self.carbon_equiv_metric_tons
    }
}

fn zero_or_more(x: &f64) -> f64 {
    if *x > 0.0 {
        *x
    } else {
        0.0
    }
}
