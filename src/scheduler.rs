use chrono::{DateTime, Utc};
use priority_queue::PriorityQueue;
use std::cmp::Eq;
use std::cmp::Ordering;
use std::hash::Hash;
use std::time::Duration;
use std::time::SystemTime;

/// Returns the current Unix-Time (number of seconds after 1970-01-01 00:00 UTC)
/*fn now() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(t) => t.as_secs(),
        Err(_) => panic!("System Time is lower than unix epoch!"),
    }
}*/

#[derive(Debug)]
struct Event<T> {
    time: DateTime<Utc>,
    item: T,
}

#[derive(Debug, Hash)]
struct PeriodicEvent<T> {
    last_scheduled: DateTime<Utc>,
    period: Duration,
    item: T,
}

struct Scheduler<S, T> {
    queue: PriorityQueue<S, DateTime<Utc>>,
    periodics: PriorityQueue<PeriodicEvent<T>, Duration>,
}

impl<S, T> Scheduler<S, T>
where
    S: Hash + Eq,
    T: Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            queue: PriorityQueue::new(),
            periodics: PriorityQueue::new(),
        }
    }

    fn housekeeping() {
        todo!()
    }

    fn add_event() {
        todo!()
    }

    fn add_periodic_event() {
        todo!()
    }
}

//
// ----- Trait implementations -----
//

impl<T> PartialEq for Event<T> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl<T> Eq for Event<T> {}

impl<T> PartialOrd for Event<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Event<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl<T> PartialEq for PeriodicEvent<T> {
    fn eq(&self, other: &Self) -> bool {
        self.last_scheduled == other.last_scheduled
    }
}

impl<T> Eq for PeriodicEvent<T> {}

impl<T> PartialOrd for PeriodicEvent<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for PeriodicEvent<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.last_scheduled.cmp(&other.last_scheduled)
    }
}
