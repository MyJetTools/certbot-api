use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::{AppContext, CertTask, TaskStatus};

#[http_route(
    method: "GET",
    route: "/api/tasks/v1/list",
    summary: "List Certificate Tasks",
    description: "Return every certificate task (issue / reissue / renew) tracked since the service started, newest first — including the ones running right now and the outcome of recent ones. Backs the UI dashboard.",
    controller: "Tasks",

    result:[
        {status_code: 200, description: "Task list", model: TasksListHttpModel},
    ]
)]
pub struct ListTasksAction {
    app: Arc<AppContext>,
}

impl ListTasksAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &ListTasksAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let tasks = action
        .app
        .list_tasks()
        .await
        .iter()
        .map(TaskHttpModel::from_task)
        .collect();

    let response = TasksListHttpModel { tasks };
    HttpOutput::as_json(response).into_ok_result(true).into()
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct TaskHttpModel {
    pub id: String,
    pub cert_name: String,
    pub domains: Vec<String>,
    pub kind: String,
    pub status: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl TaskHttpModel {
    pub fn from_task(task: &CertTask) -> Self {
        let (output, error) = match &task.status {
            TaskStatus::Running => (None, None),
            TaskStatus::Completed { output } => (Some(output.clone()), None),
            TaskStatus::Failed { error } => (None, Some(error.clone())),
        };

        TaskHttpModel {
            id: task.id.clone(),
            cert_name: task.cert_name.clone(),
            domains: task.domains.clone(),
            kind: task.kind.as_str().to_string(),
            status: task.status.as_str().to_string(),
            output,
            error,
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct TasksListHttpModel {
    pub tasks: Vec<TaskHttpModel>,
}
