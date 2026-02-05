## About

**Note:** This repo contains code that is not thought to be reusable, please do not try to use it yourself as-is.

This is a zellij plugin providing multiple features:

### Session management

A session management UI based on `skim` running in a zellij floating pane.
This should list:
- Existing zellij sessions (live first, then exited)
- Directories in `~/src` that contain a git repository, recursively.
Then, it should open sk with those items and:
- `enter` should either attach to a session or create a new one in the selected directory, naming it with the directory name
- `ctrl-d` should kill an existing session
- `ctrl-n` should create a child session to an existing one (or behave like `enter` on directories), suffixing the session name with a numeric ID to avoid collisions
- the preview should be a capture of the session for existing sessions, or a directory listing for others

### A popup task manager

The goal is to be able to run tasks inside popups.
The task should always run in an interactive shell session.

Each task is described by a command and its arguments, and is bindable to a keybind in zellij.
It will then open a floating window (or show an existing one if previously started and still running) that runs this task.
Running the binding again should hide that window, and we also provide a command that will hide all floating tasks.

## Configuration and Usage

### Building the Plugin

You need the `wasm32-wasip1` Rust target, which is configured automatically by `rust-toolchain.toml`.

```bash
cargo build --target wasm32-wasip1
```

The compiled plugin will be at `target/wasm32-wasip1/debug/zellij-ext.wasm`.

### Quick Start

Add this to your Zellij config file (usually `~/.config/zellij/config.kdl`):

```kdl
plugins {
    zellij-ext location="https://github.com/LoricAndre/zellij_ext/releases/latest/download/zellij-ext.wasm"
}

keybinds {
    shared {
        bind "Alt s" {
            MessagePlugin "zellij-ext" {
                name "session-manager"
            }
        }
        bind "Alt t" {
            MessagePlugin "zellij-ext" {
                name "toggle-task"
                payload "htop"
            }
        }
        bind "Alt h" {
            MessagePlugin "zellij-ext" {
                name "hide-all-tasks"
            }
        }
    }
}
```

Then load the plugin in your layout:

```kdl
layout {
    // ... your panes ...
    floating_panes {
        pane {
            plugin location="zellij-ext"
        }
    }
}
```

For local development, use a file path in the plugin definition:

```kdl
plugins {
    zellij-ext location="file:/path/to/zellij-ext.wasm"
}
```

### First Run: Granting Permissions

On first launch, the plugin will request permissions. You'll see a permission dialog appear. Press **'y'** or **'a'** (allow all) to grant the required permissions:

- ReadApplicationState
- ChangeApplicationState
- RunCommands
- OpenTerminalsOrPlugins
- WriteToStdin
- ReadCliPipes

Once granted, these permissions are cached and the plugin will work automatically in all future sessions.

### Features

#### Session Manager

Opens an interactive `skim` picker listing your live sessions, exited sessions, and git repositories found under `~/src`.

```kdl
bind "Alt s" {
    MessagePlugin "zellij-ext" {
        name "session-manager"
    }
}
```

Keys within the picker:
- **Enter** -- attach to an existing session, or create a new session in the selected directory
- **Ctrl+d** -- kill a live session or delete an exited session
- **Ctrl+n** -- create a child session (appends a numeric suffix to avoid name collisions)

#### Toggle Task

Runs a command in a floating popup pane. Pressing the keybind again toggles the pane's visibility without restarting the command. The command runs inside an interactive bash shell, so it persists after the initial command finishes.

```kdl
bind "Alt t" {
    MessagePlugin "zellij-ext" {
        name "toggle-task"
        payload "htop"
    }
}
```

The `payload` is the shell command to run. You can bind multiple tasks to different keys. By default the command string is used as the task identifier; to give a task an explicit ID (useful if the same command is bound with different meanings), add a `task_id` argument:

```kdl
bind "Alt g" {
    MessagePlugin "zellij-ext" {
        name "toggle-task"
        payload "lazygit"
        // optional explicit task ID
        // task_id "git"
    }
}
```

#### Hide All Tasks

Hides every visible task popup at once.

```kdl
bind "Alt h" {
    MessagePlugin "zellij-ext" {
        name "hide-all-tasks"
    }
}
```

## Releasing

To create a new release:

1. Tag a new version: `git tag v0.1.0`
2. Push the tag: `git push origin v0.1.0`

GitHub Actions will automatically build the plugin and create a release with the compiled wasm file.

## Development

*Note*: you will need to have `wasm32-wasip1` added to rust as a target to build the plugin. This is handled automatically by `rust-toolchain.toml`.

### With the Provided Layout

![img-2024-11-14-100111](https://github.com/user-attachments/assets/e3bae15c-1f94-4d4a-acea-a036f8afdf67)


Run `zellij -l zellij.kdl` at the root of this repository. This will open a development environment that will help you develop the plugin inside Zellij.

It can also be used if you prefer developing outside of the terminal - in this case you should ignore the `$EDITOR` pane and use your IDE instead.

### Otherwise

1. Build the project: `cargo build --target wasm32-wasip1`
2. Load it inside a running Zellij session: `zellij action start-or-reload-plugin file:target/wasm32-wasip1/debug/zellij-ext.wasm`
3. Repeat on changes (perhaps with a `watchexec` or similar command to run on fs changes).
