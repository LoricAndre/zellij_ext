use zellij_tile::prelude::*;

use std::collections::{BTreeMap, HashMap};

mod session_manager;
mod task_manager;

use session_manager::SessionManagerState;
use task_manager::TaskInfo;

#[derive(Default)]
struct State {
    sm: SessionManagerState,
    tasks: HashMap<String, TaskInfo>,
    got_permissions: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
            PermissionType::OpenTerminalsOrPlugins,
            PermissionType::WriteToStdin,
            PermissionType::ReadCliPipes,
        ]);
        subscribe(&[
            EventType::SessionUpdate,
            EventType::RunCommandResult,
            EventType::CommandPaneOpened,
            EventType::CommandPaneExited,
            EventType::PaneClosed,
            EventType::PermissionRequestResult,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.got_permissions = true;
                set_selectable(false);
                hide_self();
            }
            Event::SessionUpdate(sessions, resurrectable) => {
                self.sm.sessions = sessions;
                self.sm.resurrectable_sessions = resurrectable;
            }
            Event::RunCommandResult(_exit_code, stdout, _stderr, context) => {
                if context.get("purpose").map(|s| s.as_str()) == Some("scan-repos") {
                    session_manager::handle_repo_scan_result(&mut self.sm, stdout);
                }
            }
            Event::CommandPaneOpened(pane_id, context) => {
                match context.get("purpose").map(|s| s.as_str()) {
                    Some("session-manager-sk") => {
                        self.sm.sm_pane_id = Some(pane_id);
                    }
                    Some(purpose) if purpose.starts_with("task:") => {
                        let task_id = &purpose["task:".len()..];
                        task_manager::handle_pane_opened(&mut self.tasks, task_id, pane_id);
                    }
                    _ => {}
                }
            }
            Event::CommandPaneExited(pane_id, _exit_code, _context) => {
                if self.sm.sm_pane_id == Some(pane_id) {
                    self.sm.sm_pane_id = None;
                }
                task_manager::handle_pane_exited(&mut self.tasks, pane_id);
            }
            Event::PaneClosed(pane_id) => {
                if let PaneId::Terminal(id) = pane_id {
                    if self.sm.sm_pane_id == Some(id) {
                        self.sm.sm_pane_id = None;
                    }
                    task_manager::handle_pane_closed(&mut self.tasks, id);
                }
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.name.as_str() {
            "session-manager" => {
                session_manager::open_session_manager(&mut self.sm);
            }
            "session-action" => {
                session_manager::handle_session_action(&mut self.sm, &pipe_message);
            }
            "toggle-task" => {
                task_manager::toggle_task(&mut self.tasks, &pipe_message);
            }
            "hide-all-tasks" => {
                task_manager::hide_all_tasks(&mut self.tasks);
            }
            _ => {}
        }
        if let PipeSource::Cli(_) = &pipe_message.source {
            unblock_cli_pipe_input(&pipe_message.name);
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}
