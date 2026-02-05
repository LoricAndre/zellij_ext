use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;
use zellij_tile::prelude::*;

pub struct SessionManagerState {
    pub sessions: Vec<SessionInfo>,
    pub resurrectable_sessions: Vec<(String, Duration)>,
    pub repos: Vec<String>,
    pub sm_pane_id: Option<u32>,
    pub pending_repo_scan: bool,
}

impl Default for SessionManagerState {
    fn default() -> Self {
        Self {
            sessions: Vec::new(),
            resurrectable_sessions: Vec::new(),
            repos: Vec::new(),
            sm_pane_id: None,
            pending_repo_scan: false,
        }
    }
}

/// Initiate the session manager: scan for git repos, then launch sk.
pub fn open_session_manager(sm: &mut SessionManagerState) {
    if sm.sm_pane_id.is_some() || sm.pending_repo_scan {
        return;
    }
    sm.pending_repo_scan = true;
    let mut context = BTreeMap::new();
    context.insert("purpose".to_string(), "scan-repos".to_string());
    run_command(
        &[
            "sh",
            "-c",
            "find ~/src -name .git -type d -maxdepth 5 2>/dev/null",
        ],
        context,
    );
}

/// Handle the result of the find command and launch sk.
pub fn handle_repo_scan_result(sm: &mut SessionManagerState, stdout: Vec<u8>) {
    sm.pending_repo_scan = false;

    let output = String::from_utf8_lossy(&stdout);
    sm.repos = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim_end_matches("/.git").to_string())
        .collect();

    launch_sk(sm);
}

fn build_item_list(sm: &SessionManagerState) -> String {
    let mut items = Vec::new();

    // Live sessions (excluding current)
    for session in &sm.sessions {
        if !session.is_current_session {
            items.push(format!("[session] {}", session.name));
        }
    }

    // Resurrectable (exited) sessions
    for (name, _duration) in &sm.resurrectable_sessions {
        items.push(format!("[exited] {}", name));
    }

    // Git repo directories
    for repo in &sm.repos {
        items.push(format!("[dir] {}", repo));
    }

    items.join("\n")
}

fn launch_sk(sm: &mut SessionManagerState) {
    let items = build_item_list(sm);
    if items.is_empty() {
        return;
    }
    let items_escaped = items.replace('\\', "\\\\").replace('\'', "'\\''");

    let script = format!(
        concat!(
            "RESULT=$(printf '%s\\n' '{items}' | sk ",
            "--ansi ",
            "--expect 'ctrl-d,ctrl-n' ",
            "--preview '",
            "line={{}}; ",
            "type=$(echo \"$line\" | sed \"s/^\\\\[\\\\([^]]*\\\\)\\\\].*/\\\\1/\"); ",
            "name=$(echo \"$line\" | sed \"s/^\\\\[[^]]*\\\\] //\"); ",
            "if [ \"$type\" = \"session\" ]; then ",
            "echo \"Live session: $name\"; echo; zellij list-sessions 2>/dev/null | grep -A2 \"$name\" || true; ",
            "elif [ \"$type\" = \"exited\" ]; then ",
            "echo \"Exited session: $name\"; ",
            "else ",
            "ls --color=always -la \"$name\" 2>/dev/null; ",
            "fi",
            "' ",
            "--preview-window=right:50%); ",
            "KEY=$(echo \"$RESULT\" | head -1); ",
            "ITEM=$(echo \"$RESULT\" | sed -n 2p); ",
            "if [ -n \"$ITEM\" ]; then ",
            "zellij pipe --name session-action --args \"key=$KEY\" -- \"$ITEM\"; ",
            "fi; ",
            "zellij action close-pane",
        ),
        items = items_escaped,
    );

    let cmd = CommandToRun {
        path: PathBuf::from("bash"),
        args: vec!["-c".to_string(), script],
        cwd: None,
    };
    let mut context = BTreeMap::new();
    context.insert(
        "purpose".to_string(),
        "session-manager-sk".to_string(),
    );
    open_command_pane_floating(cmd, None, context);
}

/// Handle the result from sk (received as a pipe message).
pub fn handle_session_action(sm: &mut SessionManagerState, pipe_message: &PipeMessage) {
    let key = pipe_message
        .args
        .get("key")
        .map(|s| s.as_str())
        .unwrap_or("");
    let item = match pipe_message.payload.as_deref() {
        Some(i) if !i.is_empty() => i,
        _ => return,
    };

    let (item_type, item_value) = parse_item(item);

    match key {
        "" | "enter" => match item_type {
            "session" | "exited" => {
                switch_session(Some(item_value));
            }
            "dir" => {
                let name = dir_to_session_name(item_value);
                switch_session_with_cwd(Some(&name), Some(PathBuf::from(item_value)));
            }
            _ => {}
        },
        "ctrl-d" => match item_type {
            "session" => {
                kill_sessions(&[item_value]);
            }
            "exited" => {
                delete_dead_session(item_value);
            }
            _ => {}
        },
        "ctrl-n" => match item_type {
            "session" | "exited" => {
                let child_name = generate_child_name(item_value, sm);
                switch_session(Some(&child_name));
            }
            "dir" => {
                let name = dir_to_session_name(item_value);
                switch_session_with_cwd(Some(&name), Some(PathBuf::from(item_value)));
            }
            _ => {}
        },
        _ => {}
    }
}

fn parse_item(item: &str) -> (&str, &str) {
    if let Some(rest) = item.strip_prefix('[') {
        if let Some(bracket_end) = rest.find(']') {
            let item_type = &rest[..bracket_end];
            let item_value = rest[bracket_end + 1..].trim_start();
            return (item_type, item_value);
        }
    }
    ("", item)
}

fn dir_to_session_name(path: &str) -> String {
    PathBuf::from(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

fn generate_child_name(base_name: &str, sm: &SessionManagerState) -> String {
    let existing_names: Vec<&str> = sm
        .sessions
        .iter()
        .map(|s| s.name.as_str())
        .chain(sm.resurrectable_sessions.iter().map(|(n, _)| n.as_str()))
        .collect();

    for i in 1u32.. {
        let candidate = format!("{}-{}", base_name, i);
        if !existing_names.contains(&candidate.as_str()) {
            return candidate;
        }
    }
    unreachable!()
}
