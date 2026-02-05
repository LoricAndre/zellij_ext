use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use zellij_tile::prelude::*;

pub struct TaskInfo {
    pub command: String,
    pub pane_id: Option<u32>,
    pub visible: bool,
}

pub fn toggle_task(tasks: &mut HashMap<String, TaskInfo>, pipe_message: &PipeMessage) {
    let command = match pipe_message.payload.as_deref() {
        Some(cmd) if !cmd.is_empty() => cmd,
        _ => return,
    };

    let task_id = pipe_message
        .args
        .get("task_id")
        .cloned()
        .unwrap_or_else(|| command.to_string());

    if let Some(task) = tasks.get_mut(&task_id) {
        if let Some(pane_id) = task.pane_id {
            if task.visible {
                hide_pane_with_id(PaneId::Terminal(pane_id));
                task.visible = false;
            } else {
                show_pane_with_id(PaneId::Terminal(pane_id), true);
                task.visible = true;
            }
        }
        return;
    }

    let cmd = CommandToRun {
        path: PathBuf::from("bash"),
        args: vec!["-c".to_string(), format!("{}; exec bash -i", command)],
        cwd: None,
    };
    let mut context = BTreeMap::new();
    context.insert("purpose".to_string(), format!("task:{}", task_id));
    open_command_pane_floating(cmd, None, context);

    tasks.insert(
        task_id,
        TaskInfo {
            command: command.to_string(),
            pane_id: None,
            visible: true,
        },
    );
}

pub fn hide_all_tasks(tasks: &mut HashMap<String, TaskInfo>) {
    for task in tasks.values_mut() {
        if task.visible {
            if let Some(pane_id) = task.pane_id {
                hide_pane_with_id(PaneId::Terminal(pane_id));
                task.visible = false;
            }
        }
    }
}

pub fn handle_pane_opened(tasks: &mut HashMap<String, TaskInfo>, task_id: &str, pane_id: u32) {
    if let Some(task) = tasks.get_mut(task_id) {
        task.pane_id = Some(pane_id);
    }
}

pub fn handle_pane_exited(tasks: &mut HashMap<String, TaskInfo>, pane_id: u32) {
    tasks.retain(|_, task| task.pane_id != Some(pane_id));
}

pub fn handle_pane_closed(tasks: &mut HashMap<String, TaskInfo>, pane_id: u32) {
    tasks.retain(|_, task| task.pane_id != Some(pane_id));
}
