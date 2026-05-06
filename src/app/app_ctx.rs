use std::collections::HashMap;
use std::sync::Arc;

use rust_extensions::AppStates;
use tokio::sync::Mutex;

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

#[derive(Debug, Clone)]
pub enum CertJobStatus {
    Running,
    Completed { output: String },
    Failed { error: String },
}

pub enum StartRenewOutcome {
    Started,
    AlreadyRunning,
}

pub struct AppContext {
    pub app_states: Arc<AppStates>,
    pub renew_jobs: Mutex<HashMap<String, CertJobStatus>>,
}

impl AppContext {
    pub async fn new() -> Self {
        AppContext {
            app_states: Arc::new(AppStates::create_initialized()),
            renew_jobs: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_renew_job_status(&self, domain: &str) -> Option<CertJobStatus> {
        let jobs = self.renew_jobs.lock().await;
        jobs.get(domain).cloned()
    }
}

pub async fn start_renew_job(app: Arc<AppContext>, domain: String) -> StartRenewOutcome {
    let mut jobs = app.renew_jobs.lock().await;

    if let Some(CertJobStatus::Running) = jobs.get(&domain) {
        return StartRenewOutcome::AlreadyRunning;
    }

    jobs.insert(domain.clone(), CertJobStatus::Running);
    drop(jobs);

    let app_for_task = app.clone();
    tokio::spawn(async move {
        let result = crate::scripts::renew_certificate(domain.clone()).await;
        let mut jobs = app_for_task.renew_jobs.lock().await;
        let new_status = match result {
            Ok(output) => CertJobStatus::Completed { output },
            Err(error) => CertJobStatus::Failed { error },
        };
        jobs.insert(domain, new_status);
    });

    StartRenewOutcome::Started
}
