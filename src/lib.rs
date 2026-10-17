//! Manage WSL Containers sessions and run commands with time limits and process-tree cleanup.
//!
//! On Windows, `container` provides session creation, host directory mounts, and container commands.
//! The `process` module runs host commands on Windows and Unix.

#[cfg(windows)]
pub mod container;
pub mod process;
#[cfg(windows)]
pub mod sdk;
#[cfg(windows)]
pub mod session;

pub type Result<T> = std::result::Result<T, String>;
