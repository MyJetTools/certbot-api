use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

/// State of a single certificate task.
#[derive(Debug, Clone)]
pub enum TaskStatus {
    Running,
    Completed { output: String },
    Failed { error: String },
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Running => "running",
            TaskStatus::Completed { .. } => "completed",
            TaskStatus::Failed { .. } => "failed",
        }
    }
}

/// Which operation a task represents.
#[derive(Debug, Clone, Copy)]
pub enum TaskKind {
    IssueHttp01,
    ReissueHttp01,
    RenewDns,
}

impl TaskKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskKind::IssueHttp01 => "issue-http-01",
            TaskKind::ReissueHttp01 => "reissue-http-01",
            TaskKind::RenewDns => "renew-dns",
        }
    }
}

/// One certificate operation (issue / reissue / renew) tracked by the service.
/// Lives in memory only — the list resets when the service restarts, which is
/// fine: it exists to show what is running now and the outcome of recent runs.
#[derive(Debug, Clone)]
pub struct CertTask {
    pub id: String,
    /// Lineage / primary domain the task is keyed under.
    pub cert_name: String,
    /// Full domain (SAN) set the operation targets.
    pub domains: Vec<String>,
    pub kind: TaskKind,
    pub status: TaskStatus,
    pub created_at: DateTimeAsMicroseconds,
    pub updated_at: DateTimeAsMicroseconds,
}

impl CertTask {
    /// True if `domain` names this task's lineage (its `cert_name`). Both sides
    /// are normalized (a leading `*.` is stripped) so the apex and wildcard
    /// spellings of a DNS cert match the same lineage.
    ///
    /// Deliberately matches the lineage name ONLY, never the SAN set: two
    /// requests with different primary domains are different certificates, so
    /// dedup (`find_running`) and status (`latest_for`) must not conflate them
    /// just because they happen to share a secondary SAN.
    pub fn matches_domain(&self, domain: &str) -> bool {
        crate::scripts::normalize_cert_name(&self.cert_name)
            == crate::scripts::normalize_cert_name(domain)
    }
}

/// In-memory registry of certificate tasks.
#[derive(Default)]
pub struct CertTasks {
    by_id: HashMap<String, CertTask>,
    seq: u64,
}

impl CertTasks {
    pub fn new() -> Self {
        Self::default()
    }

    fn next_id(&mut self) -> String {
        self.seq += 1;
        format!("task-{}", self.seq)
    }

    /// The running task that owns this domain, if any (create-or-find dedup).
    pub fn find_running(&self, domain: &str) -> Option<&CertTask> {
        self.by_id
            .values()
            .filter(|t| matches!(t.status, TaskStatus::Running) && t.matches_domain(domain))
            .max_by_key(|t| t.created_at.unix_microseconds)
    }

    /// The most recent task (any status) for this domain, for status polling.
    pub fn latest_for(&self, domain: &str) -> Option<&CertTask> {
        self.by_id
            .values()
            .filter(|t| t.matches_domain(domain))
            .max_by_key(|t| t.created_at.unix_microseconds)
    }

    /// Insert a new task in the `Running` state and return a copy of it.
    pub fn create_running(
        &mut self,
        cert_name: String,
        domains: Vec<String>,
        kind: TaskKind,
    ) -> CertTask {
        let now = DateTimeAsMicroseconds::now();
        let id = self.next_id();
        let task = CertTask {
            id: id.clone(),
            cert_name,
            domains,
            kind,
            status: TaskStatus::Running,
            created_at: now,
            updated_at: now,
        };
        self.by_id.insert(id, task.clone());
        task
    }

    pub fn set_status(&mut self, id: &str, status: TaskStatus) {
        if let Some(task) = self.by_id.get_mut(id) {
            task.status = status;
            task.updated_at = DateTimeAsMicroseconds::now();
        }
    }

    /// All tasks, newest first.
    pub fn all_sorted(&self) -> Vec<CertTask> {
        let mut tasks: Vec<CertTask> = self.by_id.values().cloned().collect();
        tasks.sort_by(|a, b| {
            b.created_at
                .unix_microseconds
                .cmp(&a.created_at.unix_microseconds)
        });
        tasks
    }
}
