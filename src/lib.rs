pub mod settings;
// pub mod tasks;
pub mod queue_types;

#[cfg(feature = "sdr_stream")]
pub mod sdr;

use crate::observation::Observation;
use crate::queue_types::QueueJob;
use crate::settings::Settings;
use log::{debug, error, info};
use satnogs_apiclient::{
	api_client::{APIClient, BasicStationInfo},
	filters::JobFilter,
	json::Job as ApiJob,
};
use serde_json;
use std::{collections::{BinaryHeap, VecDeque}, io::Write};
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

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

/// Starts and stops observations. This function gets called once a second.
pub fn task_observation_housekeeping(
	current_job: Arc<Mutex<Option<ApiJob>>>,
	flowgraph_handle: Arc<Mutex<Option<Child>>>,
	future_jobs: Arc<Mutex<BinaryHeap<QueueJob>>>,
	conf: Settings,
) {
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
				// ToDo: This should be in a separate thread or in upload-task running in separate thread
				let _ = run_obs_scripts(
					conf.clone(),
					ObservationScript::Post,
					job.clone(),
				);
			}

			*curr_job_info = None;
		}
	}

	// --- Start next job if due ---
	// avoid starting a new one if already running
	if curr_job_info.is_some() {
		return;
	}

	if let Some(next_job) = next_job_queue.peek()
		&& next_job.start <= now
	{
		// ToDo: Das hier schick machen
		let cmd = CmdArgs {
			program: "test.py".to_string(),
			args: vec!["-a".to_string()],
			pwd: format!("{}/{}/", conf.storage.artifacts_path, next_job.job.id).into(),
		};
		let cmd2 = cmd.clone();

		let _ = run_obs_scripts(conf, ObservationScript::Pre, next_job.job.clone());

		match Command::new(cmd.program).args(cmd.args).current_dir(cmd.pwd).spawn() {
			Err(_) => {
				error!(
					"Failed to spawn flowgraph {} {:?}",
					cmd2.program, cmd2.args
				);
			},

			Ok(child) => {
				info!("Started flowgraph: {} {:?}", cmd2.program, cmd2.args);

				*curr_job_handle = Some(child);

				// move job from queue -> current_job
				*curr_job_info = next_job_queue.pop().map(|qj| qj.job);
			},
		}
	}
}

enum ObservationScript {
	Pre,
	Post,
}

/// Run Pre/Post Observation scripts in lexicographical order. For that, we
/// change directory to /tmp/satnogs-rs/$observation_id/
fn run_obs_scripts(conf: Settings, pre_post: ObservationScript, job: ApiJob) -> anyhow::Result<()> {
	let (enabled, script_path, script_type) = match pre_post {
		ObservationScript::Pre => {
			(conf.scripting.pre_observation_script, &conf.scripting.pre_obs_path, "pre")
		},
		ObservationScript::Post => (
			conf.scripting.post_observation_script,
			&conf.scripting.post_obs_path,
			"post",
		),
	};

	if !enabled {
		return Ok(());
	}

	debug!("Creating observation '{}' directory.", job.id);
	let cwd = format!("{}/{}", conf.storage.artifacts_path, job.id);
	fs::create_dir_all(&cwd)?;

	debug!("Saving job information as json...");
	let job_json_path = format!("{}/{}", cwd, "job.json");
	let mut f = fs::File::create(job_json_path)?;
	let json = serde_json::to_string(&job)?;
	match f.write(&json.into_bytes()) {
		Ok(bytes) => debug!("Written {bytes} bytes."),
		Err(e) => error!("Writing job json failed: {}", e),
	}


	let mut entries: Vec<_> = fs::read_dir(script_path)?.filter_map(Result::ok).collect();
	entries.sort_by_key(|e| e.path());

	for entry in entries {
		let path = entry.path();
		if !path.is_file() {
			continue;
		}

		match Command::new(&path).current_dir(&cwd).spawn() {
			Ok(mut child) => {
				info!("Running {}-script: '{:?}'", script_type, path);
				child.wait()?;
			},
			Err(e) => {
				error!("Running {}-script '{:?}' failed: {}", script_type, path, e);
				error!("Did you set the exec-bit for the script?")
			},
		}
	}

	Ok(())
}

pub fn task_upload_artifacts(upload_queue: Arc<Mutex<BinaryHeap<QueueJob>>>) -> anyhow::Result<()> {
	debug!("Checking for artifacts to be uploaded...");

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
		None => {
			debug!("Nothing to be uploaded.");
			return Ok(());
		},
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
) -> anyhow::Result<()> {
	debug!("Polling Network for jobs...");
	let mut future_jobs = q_future_jobs.lock().unwrap();
	debug!("Acquired future_jobs mutex.");

	future_jobs.clear();
	let jobs = fetch_jobs(api.clone(), settings.clone())?;
	for job in jobs {
		future_jobs.push(QueueJob { start: job.start, job });
	}
	debug!("{} future jobs in queue.", future_jobs.len());

	Ok(())
}

pub fn fetch_jobs(api: APIClient, conf: Settings) -> anyhow::Result<Vec<ApiJob>> {
	let f = JobFilter {
		ground_station: Some(conf.station.id),
		..Default::default()
	};

	let station_info = BasicStationInfo {
		ground_station: conf.station.id,
		lat: conf.station.lat,
		lon: conf.station.lon,
		alt: conf.station.alt,
	};

	Ok(api.get_jobs_heartbeat(f, station_info, conf.network.token)?)
}

#[derive(Debug, Clone)]
pub struct CmdArgs {
	pub program: String,
	pub args: Vec<String>,
	pub pwd: PathBuf,
}

mod test {
	use chrono::{DateTime, Utc};

	use super::*;

	#[test]
	fn pre_post_scripts() {
		let conf = Settings::new().unwrap();
		let job = ApiJob {
			id: 42,
			start: DateTime::parse_from_rfc3339("2026-01-31T00:00:00Z")
				.unwrap()
				.with_timezone(&Utc),
			end: DateTime::parse_from_rfc3339("2026-01-31T00:02:00Z")
				.unwrap()
				.with_timezone(&Utc),
			ground_station: 42,
			tle0: "".to_string(),
			tle1: "".to_string(),
			tle2: "".to_string(),
			frequency: 430200000,
			mode: Some("GMSK".to_string()),
			transmitter: "abf4329bdee".to_string(),
			baud: Some(9600.0),
			max_altitude: 80.0,
			norad_cat_id: 42000,
		};
		let _ = run_obs_scripts(conf.clone(), ObservationScript::Pre, job.clone());
		let _ = run_obs_scripts(conf, ObservationScript::Post, job);

		// assert job json
		// assert
	}
}
