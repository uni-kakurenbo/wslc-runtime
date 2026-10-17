#![cfg(windows)]

use std::path::Path;
use std::process::Command;
use wslc_runtime::container::{Context, Pull, RunOptions};

#[test]
fn borrowed_connections_only_construct_commands_and_export_their_identity() {
    // No SDK or session is created: the current executable is a stand-in CLI.
    let program = std::env::current_exe().unwrap();
    let context = Context::connect(Path::new("."), "borrowed-session", Some(&program)).unwrap();
    let command = context
        .command(&RunOptions {
            image: "example/image:version".into(),
            mounts: Vec::new(),
            workdir: "/".into(),
            entrypoint: "/bin/true".into(),
            arguments: Vec::new(),
            pull: Pull::Never,
            interactive: false,
            tty: false,
        })
        .unwrap();
    let args: Vec<_> = command.get_args().collect();
    assert_eq!(args[0], "--session");
    assert_eq!(args[1], "borrowed-session");

    let mut child = Command::new(&program);
    context.configure_child(&mut child);
    assert!(child.get_envs().any(|(key, value)| key == "WSLC_SESSION"
        && value.is_some_and(|value| value == "borrowed-session")));
    assert!(Context::connect(Path::new("."), "", Some(&program)).is_err());
}
