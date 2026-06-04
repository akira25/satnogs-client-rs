use crate::settings::Settings;
use log::{error, info};
use serde::{Deserialize, Serialize};
use ureq;
use crate::AppState;

// #[derive(Serialize, Deserialize, Debug, Clone,)]
// pub struct Job {
//     id: u64,
//     start: String,
//     end: String,
//     ground_station: u32,
//     tle0: String,
//     tle1: String,
//     tle2: String,
//     frequency: u64,
//     mode: String,
//     transmitter: String,
//     baud: Option<f32>,
//     max_altitude: f32,
//     norad_cat_id: u32,
// }

pub fn start_job() {
    todo!()
}

pub fn stop_job() {
    todo!()
}

pub fn fetch_jobs(agent: ureq::Agent, app: &mut AppState) {
    info!("Fetching jobs from Network...");
    let url = app.settings.network.url.clone() + "jobs/{}";

    // SatNOGs Network will update the groundstation position from the parameters given here.
    let mut resp = agent
        .get(url)
        // .header("Authorization", format!("Token {0}", conf.network.token))
        .query("ground_station", format!("{}", app.settings.station.id))
        .query("lat", format!("{:.5}", app.settings.station.lat))
        .query("lon", format!("{:.5}", app.settings.station.lon))
        .query("alt", format!("{}", app.settings.station.alt))
        .call()
        .unwrap();

    if !resp.status().is_success() {
        error!("Fetching jobs from network failed!");
        return;
    }

    let fetched_jobs = resp.body_mut().read_json::<Vec<Job>>().expect("Parsing the Jobs-JSON failed!");
    info!("Fetched {} jobs from network.", fetched_jobs.len());

    return;
}

pub fn upload_artifacts() {}
