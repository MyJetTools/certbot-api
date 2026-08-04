use std::sync::Arc;

use rust_extensions::AppStates;
use tokio::sync::Mutex;

use crate::scripts::CertbotPrepareError;

use super::{CertTask, CertTasks, TaskKind, TaskStatus};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub enum StartTaskOutcome {
    Started,
    AlreadyRunning,
}

/// Result of starting a certificate task: whether a new task was spawned or an
/// existing one was found, plus the task itself (id, cert_name, status, …).
pub struct StartedTask {
    pub outcome: StartTaskOutcome,
    pub task: CertTask,
}

pub struct AppContext {
    pub app_states: Arc<AppStates>,
    pub tasks: Mutex<CertTasks>,
}

impl AppContext {
    pub async fn new() -> Self {
        AppContext {
            app_states: Arc::new(AppStates::create_initialized()),
            tasks: Mutex::new(CertTasks::new()),
        }
    }

    /// Latest task (any status) for a domain — matched by lineage name or any
    /// SAN. Used by the status endpoints.
    pub async fn get_task_for_domain(&self, domain: &str) -> Option<CertTask> {
        self.tasks.lock().await.latest_for(domain).cloned()
    }

    /// All tasks, newest first — used by the tasks list endpoint / UI.
    pub async fn list_tasks(&self) -> Vec<CertTask> {
        self.tasks.lock().await.all_sorted()
    }
}

/// Shared create-or-find + spawn skeleton. `prepared` is already validated; the
/// task is keyed by `cert_name` and, if a task for it is already running, that
/// running task is returned instead of starting a second certbot run.
async fn start_task<Fut>(
    app: Arc<AppContext>,
    cert_name: String,
    domains: Vec<String>,
    kind: TaskKind,
    run: Fut,
) -> StartedTask
where
    Fut: std::future::Future<Output = Result<String, String>> + Send + 'static,
{
    let mut tasks = app.tasks.lock().await;

    if let Some(existing) = tasks.find_running(&cert_name) {
        return StartedTask {
            outcome: StartTaskOutcome::AlreadyRunning,
            task: existing.clone(),
        };
    }

    let task = tasks.create_running(cert_name, domains, kind);
    let id = task.id.clone();
    drop(tasks);

    let app_for_task = app.clone();
    tokio::spawn(async move {
        let status = match run.await {
            Ok(output) => TaskStatus::Completed { output },
            Err(error) => TaskStatus::Failed { error },
        };
        app_for_task.tasks.lock().await.set_status(&id, status);
    });

    StartedTask {
        outcome: StartTaskOutcome::Started,
        task,
    }
}

/// Start a background HTTP-01 issuance task. Cheap input validation and the
/// no-clobber guard run synchronously (a bad request fails right here); only the
/// certbot call runs in the spawned task.
pub async fn start_init_http_01_task(
    app: Arc<AppContext>,
    domains: Vec<String>,
    email: String,
    webroot: Option<String>,
) -> Result<StartedTask, CertbotPrepareError> {
    let prepared = crate::scripts::prepare_init_http_01(domains, email, webroot).await?;
    let cert_name = prepared.cert_name.clone();
    let task_domains = prepared.domains.clone();

    Ok(start_task(
        app,
        cert_name,
        task_domains,
        TaskKind::IssueHttp01,
        crate::scripts::run_init_http_01(prepared),
    )
    .await)
}

/// Start a background HTTP-01 reissue task.
pub async fn start_reissue_http_01_task(
    app: Arc<AppContext>,
    domain: String,
) -> Result<StartedTask, CertbotPrepareError> {
    let prepared = crate::scripts::prepare_reissue_http_01(domain).await?;
    let cert_name = prepared.cert_name.clone();

    Ok(start_task(
        app,
        cert_name.clone(),
        vec![cert_name],
        TaskKind::ReissueHttp01,
        crate::scripts::run_reissue_http_01(prepared),
    )
    .await)
}

/// Start a background DNS-01 renewal task (apex + wildcard).
pub async fn start_renew_dns_task(
    app: Arc<AppContext>,
    domain: String,
) -> Result<StartedTask, CertbotPrepareError> {
    let prepared = crate::scripts::prepare_renew_dns(domain).await?;
    let cert_name = prepared.cert_name.clone();
    let task_domains = prepared.domains.clone();

    Ok(start_task(
        app,
        cert_name,
        task_domains,
        TaskKind::RenewDns,
        crate::scripts::run_renew_dns(prepared),
    )
    .await)
}
