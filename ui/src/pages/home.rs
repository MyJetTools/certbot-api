use std::time::Duration;

use dioxus::prelude::*;

use crate::components::Topbar;
use crate::components::atoms::{StatePill, StateTone};
use crate::models::{DomainModel, TaskModel};

/// One struct, one signal for the whole page.
#[derive(Default)]
pub struct HomeState {
    domains: Vec<DomainModel>,
    tasks: Vec<TaskModel>,
}

impl HomeState {
    /// Applies one poll tick. Each half is optional: a fetch that failed keeps
    /// the last known value instead of blanking the table.
    fn apply_tick(&mut self, domains: Option<Vec<DomainModel>>, tasks: Option<Vec<TaskModel>>) {
        if let Some(domains) = domains {
            self.domains = domains;
        }
        if let Some(tasks) = tasks {
            self.tasks = tasks;
        }
    }
}

/// One tick of the poller: both endpoints.
async fn poll_once(mut cs: Signal<HomeState>) {
    let domains = match crate::api::get_domains().await {
        Ok(domains) => Some(domains),
        Err(err) => {
            dioxus_utils::console_log(format!("Domains poll failed: {}", err));
            None
        }
    };

    let tasks = match crate::api::get_tasks().await {
        Ok(tasks) => Some(tasks),
        Err(err) => {
            dioxus_utils::console_log(format!("Tasks poll failed: {}", err));
            None
        }
    };

    cs.write().apply_tick(domains, tasks);
}

fn domain_tone(domain: &DomainModel) -> StateTone {
    if domain.is_error() {
        StateTone::Bad
    } else {
        StateTone::Ok
    }
}

fn task_tone(status: &str) -> StateTone {
    match status {
        "running" => StateTone::Warn,
        "completed" => StateTone::Ok,
        "failed" => StateTone::Bad,
        _ => StateTone::Neutral,
    }
}

#[component]
pub fn Home() -> Element {
    let cs = use_signal(HomeState::default);

    // One poller drives both tables. `use_hook` runs its body exactly once per
    // component instance, so the loop starts once without writing to a signal
    // during render.
    use_hook(move || {
        spawn(async move {
            loop {
                poll_once(cs).await;
                dioxus_utils::js::sleep(Duration::from_secs(5)).await;
            }
        });
    });

    let cs_ra = cs.read();

    let domains_count = cs_ra.domains.len();
    let running_count = cs_ra.tasks.iter().filter(|t| t.is_running()).count();
    let tasks_count = cs_ra.tasks.len();

    let domain_rows: Vec<Element> = cs_ra
        .domains
        .iter()
        .map(|d| {
            rsx! {
                tr { key: "{d.domain}",
                    td { class: "mono", "{d.domain}" }
                    td { class: "mono", "{d.expiration_label()}" }
                    td {
                        StatePill {
                            label: d.status_label().to_string(),
                            tone: domain_tone(d),
                            title: d.error_title().map(|t| t.to_string()),
                        }
                    }
                }
            }
        })
        .collect();

    let domains_body = if domain_rows.is_empty() {
        rsx! {
            tr {
                td { class: "dt__empty", colspan: "3", "No domains served yet." }
            }
        }
    } else {
        rsx! {
            {domain_rows.into_iter()}
        }
    };

    let task_rows: Vec<Element> = cs_ra
        .tasks
        .iter()
        .map(|t| {
            rsx! {
                tr { key: "{t.id}",
                    td { class: "mono", "{t.time_label()}" }
                    td { class: "mono", "{t.kind}" }
                    td { class: "mono", "{t.cert_name}" }
                    td {
                        span { class: "mono dt-ellipsis", title: "{t.domains_label()}", "{t.domains_label()}" }
                    }
                    td {
                        StatePill {
                            label: t.status.clone(),
                            tone: task_tone(&t.status),
                            title: t.status_title(),
                        }
                    }
                }
            }
        })
        .collect();

    let tasks_body = if task_rows.is_empty() {
        rsx! {
            tr {
                td { class: "dt__empty", colspan: "5", "No tasks yet." }
            }
        }
    } else {
        rsx! {
            {task_rows.into_iter()}
        }
    };

    rsx! {
        div { class: "shell",
            Topbar {}
            section { class: "page page--padded",
                div { style: "display: flex; flex-direction: column; gap: 14px;",

                    // ----- Served domains -----
                    div { class: "card",
                        div { class: "card__header",
                            span { class: "card__title", "Served domains" }
                            span { class: "card__subtitle", "{domains_count} total" }
                        }
                        table { class: "dt",
                            thead {
                                tr {
                                    th { "Domain" }
                                    th { "Expires" }
                                    th { "Status" }
                                }
                            }
                            tbody { {domains_body} }
                        }
                    }

                    // ----- Tasks -----
                    div { class: "card",
                        div { class: "card__header",
                            span { class: "card__title", "Tasks" }
                            span { class: "card__subtitle", "{running_count} running · {tasks_count} total" }
                        }
                        table { class: "dt",
                            thead {
                                tr {
                                    th { "Time" }
                                    th { "Kind" }
                                    th { "Certificate" }
                                    th { "Domains" }
                                    th { "Status" }
                                }
                            }
                            tbody { {tasks_body} }
                        }
                    }
                }
            }
        }
    }
}
