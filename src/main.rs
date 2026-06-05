use clap::Parser;
use clokwerk::{Job, Scheduler, TimeUnits};
use log::{debug, error, info};
use satnogs_apiclient::{api_client::APIClient, json::Job as ApiJob};
use satnogs_client_rs::*;
use satnogs_client_rs::{queue_types::QueueJob, settings::Settings};
use signal_hook::{
	consts::{SIGINT, SIGTERM},
	iterator::Signals,
};
use simplelog::{LevelFilter, SimpleLogger};
use std::{
	collections::BinaryHeap,
	fs,
	process::Child,
	sync::{Arc, Mutex},
	time::Duration,
};
use ureq::Agent;

#[derive(Debug, Parser)]
struct Cli {
	#[arg(long, help = "Set logging level (debug, info, warn, error). Default is 'info'")]
	log: Option<log::LevelFilter>,
}

fn main() -> anyhow::Result<()> {
	let args = Cli::parse();
	let _logger = SimpleLogger::init(
		args.log.unwrap_or(LevelFilter::Info),
		simplelog::Config::default(),
	);
	let mut signals = Signals::new([SIGTERM])?;

	//
	// --- global state ---
	//
	// ToDo: Maybe get rid of the many arc-mutexes?
	let future_jobs: Arc<Mutex<BinaryHeap<QueueJob>>> = Arc::new(Mutex::new(BinaryHeap::new()));
	let current_job: Arc<Mutex<Option<ApiJob>>> = Arc::new(Mutex::new(None));
	let upload_queue: Arc<Mutex<BinaryHeap<QueueJob>>> =
		Arc::new(Mutex::new(BinaryHeap::new()));
	let flowgraph_handle: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

	info!("Loading settings...");
	let settings = Settings::new()?;
	debug!("Settings: {:?}", settings);

	debug!("Creating run directory.");
	fs::create_dir_all(settings.clone().storage.artifacts_path)?;

	debug!("Initialising API-Client..");
	let agent = Agent::config_builder().timeout_global(Some(Duration::from_secs(20)));
	let api = APIClient::new(agent, settings.network.url.clone());
	debug!("done.");

	// fetch jobs and schedule them initially
	info!(
		"Fetching jobs for station {} from {}",
		settings.station.id, settings.network.url
	);
	let jobs = fetch_jobs(api.clone(), settings.clone())?;
	info!("Fetched {} jobs from the network", jobs.len());

	debug!("Trying to acquire lock...");
	let mut queue = future_jobs.lock().unwrap();
	debug!("Acquired lock!");
	for job in jobs {
		debug!("Adding job {}", job.id);
		queue.push(QueueJob { start: job.start, job });
	}
	debug!("Content of future_jobs: {:?}", queue);
	drop(queue);

	info!("Initialising Scheduler...");
	let mut scheduler = Scheduler::with_tz(chrono::Utc);

	// API load balancing trough random offset. distributes client accesses
	// over the minute
	let offset = rand::random_range(5..=10);

	// --- Fetch current job list from network every 2 minutes ---
	let conf = settings.clone();
	let qfuture_jobs = Arc::clone(&future_jobs);
	scheduler
		.every(settings.network.query_interval.seconds())
		.plus(offset.seconds())
		.run(move || {
			let res = task_poll_network_jobs(
				Arc::clone(&qfuture_jobs),
				api.clone(),
				conf.clone(),
			);
			match res {
				Ok(_) => {},
				Err(e) => error!("{}", e),
			}
		});
	debug!("... registered network polling jobs.");

	// --- Upload artifacts (if any) every minute ---
	scheduler
		.every(settings.network.post_interval.seconds())
		.plus(offset.seconds())
		.run(move || {
			let _ = task_upload_artifacts(Arc::clone(&upload_queue));
		});
	info!("... registered upload task.");

	// --- poll for starting/stopping a job ---
	let conf = settings.clone();
	let hfuture_jobs = future_jobs.clone();
	scheduler.every(1.second()).run(move || {
		task_observation_housekeeping(
			Arc::clone(&current_job),
			Arc::clone(&flowgraph_handle),
			Arc::clone(&hfuture_jobs),
			conf.clone(),
		);
	});

	let scheduler_handle = scheduler.watch_thread(Duration::from_millis(500));

	// endless iterator over signals. Effectively this is a loop{}
	info!("Setup complete.");
	for sig in signals.forever() {
		info!("Received signal {:?}", sig);
		if sig == SIGTERM || sig == SIGINT {
			// stop child processes and scheduler
			break;
		}
	}

	info!("Stopping sub processes...");
	// ToDo: stop child processes
	scheduler_handle.stop();

	info!("Exiting.");
	Ok(())
}
