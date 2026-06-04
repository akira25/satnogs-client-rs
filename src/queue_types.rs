use chrono::{DateTime, Utc};
use satnogs_apiclient::json::Job;

use std::cmp::Ordering;

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]

#[derive(Debug, Clone)]
pub struct QueueJob {
    pub start: DateTime<Utc>,
    pub job: Job,
}

impl PartialEq for QueueJob {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
    }
}

impl Eq for QueueJob {}

impl PartialOrd for QueueJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // reverse ordering -> earliest date first
        other.start.cmp(&self.start)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_job_0() -> Job {
        Job {
            id: 42,
            start: DateTime::from_timestamp(1779627866, 0).unwrap(),
            end: DateTime::from_timestamp(1779627886, 0).unwrap(),
            ground_station: 106,
            tle0: "TLE".to_string(),
            tle1: "0".to_string(),
            tle2: "1".to_string(),
            frequency: 435950000,
            mode: Some("FM".to_string()),
            transmitter: "xyz".to_string(),
            baud: Some(9600.0),
            max_altitude: 45.0,
            norad_cat_id: 42000,
        }
    }

    fn create_job_1() -> Job {
        Job {
            id: 42,
            start: DateTime::from_timestamp(1779627876, 0).unwrap(),
            end: DateTime::from_timestamp(1779627896, 0).unwrap(),
            ground_station: 106,
            tle0: "TLE".to_string(),
            tle1: "0".to_string(),
            tle2: "1".to_string(),
            frequency: 435950000,
            mode: Some("FM".to_string()),
            transmitter: "xyz".to_string(),
            baud: Some(9600.0),
            max_altitude: 45.0,
            norad_cat_id: 42000,
        }
    }

    #[test]
    fn jobs_lt() {
        let job_0 = create_job_0();
        let job_1 = create_job_1();
        assert!(job_0 < job_1);
    }

    #[test]
    fn jobs_le() {
        let job_0 = create_job_0();
        let job_1 = create_job_1();
        assert!(job_0 < job_1);
    }
}
