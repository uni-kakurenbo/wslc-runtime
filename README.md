# wslc-runtime

Run Linux containers through WSL Containers (WSLC), manage their sessions, and share a session with host commands. The Rust API also runs host processes with time limits and process-tree cleanup.

## Install

Container execution requires x64 Windows and WSL 2.9.3 or later. Build with the repository's Rust 1.99.0 toolchain. The first session downloads Microsoft.WSL.Containers 3.0.1 and verifies its SHA-256 checksum.

```sh
cargo install --git https://github.com/uni-kakurenbo/wslc-runtime.git --locked wslc-runtime
wslc-session --help
```

To install from a checkout, run `cargo install --path . --locked`.

The `process` module supports Windows and Unix. The SDK, session, and container APIs are available on Windows.

## Share a session with a command

```sh
wslc-session -- your-build-tool
```

The launcher creates a session, sets `WSLC_SESSION` and `WSLC_EXECUTABLE` for the command, and keeps the session alive until the command finishes. Child processes use these variables to connect to the same session. Nested launchers reuse an existing `WSLC_SESSION`.

Standard input, output, error, and the command's exit status pass through to the caller. Session startup and cleanup run without routine log messages on those streams.

## Use the Rust API

Use `with_session` to create a session for an operation and release it afterward:

```rust,no_run
use std::process::Command;
use wslc_runtime::container::{SessionOptions, with_session};
use wslc_runtime::process::{Runner, CONTAINER_TIMEOUT};

let root = std::env::current_dir().unwrap();
let options = SessionOptions::new(root.join(".tools"));
with_session(&root, &options, |session| {
    let mut command = Command::new("your-build-tool");
    session.configure_child(&mut command);
    Runner { root: &root }.run_with_input(command, CONTAINER_TIMEOUT, false, true)?;
    Ok(())
}).unwrap();
```

`configure_child` sets the connection variables on a `Command`. Use `child_environment` when another process-launching API accepts environment entries. The session owner must remain alive until its children finish.

`with_connection` connects to `WSLC_SESSION` when set and creates a session otherwise. `Context::connect` connects to a named session managed by another owner; that owner remains responsible for its lifetime.

## Configuration and parallel execution

| Environment variable | Purpose | Default |
| --- | --- | --- |
| `WSLC_SESSION` | Existing session to connect to | Create a session when unset |
| `WSLC_EXECUTABLE` | Path to `wslc.exe` | Search PATH, then Program Files/WSL |
| `WSLC_CACHE` | SDK cache directory | LocalAppData/wslc-runtime; temporary directory if LocalAppData is unavailable |
| `WSLC_STORAGE` | Storage for a session created by this process | Cache directory/session-storage |
| `WSLC_MEMORY_MB` | Session memory limit in MB | 2048 |
| `WSLC_SDK_VERSION` / `WSLC_SDK_SHA256` | SDK archive version and expected checksum | Pinned 3.0.1 archive and checksum |
| `WSLC_COMMAND_TIMEOUT` | Launcher command timeout in seconds | 1800 |

In Rust, `SessionOptions::from_env` reads the SDK, executable, storage, and memory settings. `with_connection` reads `WSLC_SESSION`, and `WSLC_COMMAND_TIMEOUT` applies to the launcher. You can also set `SessionOptions` fields explicitly. CPU count follows the host's available parallelism. The supported SDK C ABI is 3.0.1.

Sessions sharing a storage directory run one at a time. To run containers in parallel, keep one session alive and let child commands connect to it. To create multiple sessions concurrently, give each owner a separate `SessionOptions.storage` directory. SDK downloads use a separate cache lock.

## Run containers and host processes

`RunOptions` specifies the image, mounts, container working directory, entry point, and arguments. Use `Context::command` to construct a command or `Context::run` to execute it. Mount sources must be local Windows drive paths, and container destinations must be absolute Linux paths. UNC mount sources are unsupported.

`Runner` manages a Windows Job Object or Unix process group and terminates remaining descendants on completion or timeout. Captured output is decoded as UTF-8, replacing invalid sequences. Use inherited standard handles to forward bytes unchanged. `stdin_available` checks whether Windows input can be attached to WSLC; NUL input supplies EOF.

Owned sessions are terminated and released on normal completion, errors, and command timeouts. Killing the owner process itself can prevent SDK cleanup. The creator of a borrowed session is responsible for terminating it.

## Verify

```sh
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
```

The tests cover process arguments, captured output, cache integrity, session cleanup, and timeouts. Run the console inheritance test from an attached Windows terminal:

```sh
cargo test --locked --test process -- --ignored --exact streaming_children_keep_the_attached_console --nocapture
```

Container integration tests are in [wslc-gcc](https://github.com/uni-kakurenbo/wslc-gcc).

[MIT License](LICENSE).
