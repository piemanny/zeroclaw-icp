use crate::sop;
use crate::tools::result::ToolResult;
use ic_cdk::api::time;
use std::collections::HashMap;

const COST: u64 = 2_000_000;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub cron_expr: String,
    pub payload: String,
    pub next_run: u64,
    pub enabled: bool,
}

impl ScheduledTask {
    pub fn new(id: String, cron_expr: String, payload: String, next_run: u64) -> Self {
        ScheduledTask {
            id,
            cron_expr,
            payload,
            next_run,
            enabled: true,
        }
    }
}

thread_local! {
    static TASK_QUEUE: std::cell::RefCell<HashMap<String, ScheduledTask>> =
        std::cell::RefCell::new(HashMap::new());
}

pub fn schedule_task(id: String, cron_expr: String, payload: String) -> ToolResult {
    let interval = sop::parse_cron_expr(&cron_expr).unwrap_or(60);
    let next_run = time() + interval;

    let task = ScheduledTask::new(id.clone(), cron_expr.clone(), payload, next_run);

    TASK_QUEUE.with(|q| {
        q.borrow_mut().insert(id.clone(), task);
    });

    let _ = sop::add_sop(id, cron_expr, String::new(), String::new());

    ToolResult::ok("schedule_task", "Task scheduled successfully", COST)
}

pub fn cancel_task(id: &str) -> ToolResult {
    TASK_QUEUE.with(|q| {
        q.borrow_mut().remove(id);
    });

    let _ = sop::remove_sop(id);

    ToolResult::ok("cancel_task", "Task cancelled", COST)
}

pub fn list_tasks() -> Vec<ScheduledTask> {
    TASK_QUEUE.with(|q| {
        q.borrow().values().cloned().collect()
    })
}

pub fn get_task(id: &str) -> Option<ScheduledTask> {
    TASK_QUEUE.with(|q| {
        q.borrow().get(id).cloned()
    })
}
