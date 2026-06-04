pub mod observation;
pub mod settings;
// pub mod tasks;
// pub mod scheduler;
pub mod queue_types;
// pub mod tmp;

use crate::observation::Observation;
use crate::settings::Settings;
use crate::queue_types::QueueJob;
use std::collections::VecDeque;
use std::process::Child;
use std::sync::{Arc, Mutex};
use log::{debug, error, info};
use satnogs_apiclient::{api_client::APIClient, filters::JobFilter, json::Job as ApiJob};
use std::collections::BinaryHeap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct AppState {
	pub settings: Settings,
	pub future_jobs: BinaryHeap<QueueJob>,
	pub current_job: Option<ApiJob>,
	pub upload_queue: VecDeque<Observation>,
	pub api: APIClient,
	pub flowgraph_handle: Arc<Mutex<Option<Child>>>,
}

impl AppState {
	pub fn new(api: APIClient) -> Self {
		AppState {
			settings: Settings::new().unwrap(),
			future_jobs: BinaryHeap::new(),
			current_job: None,
			upload_queue: VecDeque::new(),
			flowgraph_handle: Arc::new(Mutex::new(None)),
			api,
		}
	}
}

pub fn task_housekeeping(
	current_job: Arc<Mutex<Option<ApiJob>>>,
	flowgraph_handle: Arc<Mutex<Option<Child>>>,
	future_jobs: Arc<Mutex<BinaryHeap<QueueJob>>>,
    conf: Settings,
) {
	debug!("Polling local job queue.");
	let now = chrono::Utc::now();

	// Try to acquire locks on queues etc
	let mut curr_job_info = match current_job.try_lock() {
		Ok(lock) => lock,
		Err(_) => return,
	};
	let mut curr_job_handle = match flowgraph_handle.try_lock() {
		Ok(lock) => lock,
		Err(_) => return,
	};
	let mut next_job_queue = match future_jobs.try_lock() {
		Ok(lock) => lock,
		Err(_) => return,
	};

	// --- End running job if needed ---
	if let Some(job) = curr_job_info.as_mut() {
		if job.end <= now {
			// stop running flowgraph
			if let Some(child) = curr_job_handle.as_mut() {
				let _ = child.kill();
				let _ = child.wait();
				info!("Stopped flowgraph: 'ToDo'");
			}

			// current job finished
			*curr_job_info = None;
		}
	}

	// --- Start next job if due ---
	// avoid starting a new one if already running
	if curr_job_info.is_some() {
		return;
	}

	if let Some(next_job) = next_job_queue.peek() {
		if next_job.start <= now {
			let cmd = CmdArgs {
				program: "python3".to_string(),
				args: vec!["./test.py".to_string(), "-a".to_string()],
                pwd: conf.storage.app_path.into(),
			};

			match Command::new(cmd.program).args(cmd.args).spawn() {
				Err(_) => {
					error!("Failed to spawn flowgraph 'ToDO'!");
				},

				Ok(child) => {
					info!("Started flowgraph: 'ToDo'");

					// save process handle
					*curr_job_handle = Some(child);

					// move job from queue -> current_job
					*curr_job_info = next_job_queue.pop().map(|qj| qj.job);
				},
			}
		}
	}
}

/// Changes to dir /tmp/satnogs-rs/$observation_id/ and executes post-scripts in lexicographical order
fn run_post_scripts(conf: Settings) {
	if conf.scripting.post_observation_script {
		let scripts = fs::read_dir(conf.scripting.post_obs_path).unwrap();
		for script in scripts {
			todo!();
		}
	}
}

pub fn task_upload_artifacts(upload_queue: Arc<Mutex<BinaryHeap<QueueJob>>>) -> anyhow::Result<()> {
	debug!("Checking for artifacts to be uploaded...");
	debug!("Nothing to be uploaded.");

	let mut upload_queue = upload_queue.lock().unwrap();
	debug!("Acquired upload_queue mutex.");

	match upload_queue.peek() {
		Some(qj) => {
			info!("Start uploading artifacts for job {}", qj.job.id);
			let j = qj.job.clone();
			// todo!("Stuff for upload tasks");
			match upload_job_artifacts(j) {
				Ok(_) => {
					info!("Upload complete! Deleting files on disk....");
					// ToDo: Delete function
					info!("Deletion complete.");
				},
				Err(e) => {
					info!("Upload failed! Artifacts remain on disk.");
					return Err(e);
				},
			}

			// Upload was successful
			let _ = upload_queue.pop();
		},
		None => return Ok(()),
	};

	Ok(())
}

fn upload_job_artifacts(job: ApiJob) -> anyhow::Result<()> {
	Ok(())
}

pub fn task_poll_network_jobs(
	q_future_jobs: Arc<Mutex<BinaryHeap<QueueJob>>>,
	api: APIClient,
	settings: Settings,
) {
	debug!("Polling Network for jobs...");
	let mut future_jobs = q_future_jobs.lock().unwrap();
	debug!("Acquired future_jobs mutex.");

	future_jobs.clear();
	let jobs = fetch_jobs(api.clone(), settings.station.id);
	for job in jobs {
		future_jobs.push(QueueJob { start: job.start, job });
	}
	debug!("{} future jobs in queue.", future_jobs.len());
}

pub fn fetch_jobs(api: APIClient, station: u32) -> Vec<ApiJob> {
	let f = JobFilter {
		ground_station: Some(station),
		..Default::default()
	};

	api.get_jobs(f).unwrap()
}

pub struct CmdArgs {
	pub program: String,
	pub args: Vec<String>,
    pub pwd: PathBuf,
}

/// Called every second and starts an observation, if on time
fn observation_start() {
	todo!()
}

/// Called every second and stops observation on time
fn observation_stop() {
	todo!()
}

fn flowgraph_task() {
	let script = "./test.py";
	let handle = Command::new("python3").arg(script).spawn();

	match handle {
		Err(_) => {
			error!("Failed to spawn flowgraph '{script}'!")
		},
		Ok(h) => {
			info!("Started flowgraph: '{script}'");
			println!("{:?}", h);
		},
	}

	// return handle;
}
