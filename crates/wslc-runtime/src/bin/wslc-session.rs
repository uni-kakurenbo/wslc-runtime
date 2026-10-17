//! Run a host command with a WSLC session available to its child processes.

use std::process::ExitCode;

#[cfg(windows)]
fn run() -> wslc_runtime::Result<i32> {
    use std::path::Path;
    use std::time::Duration;
    use wslc_runtime::container::{SessionOptions, with_connection};
    use wslc_runtime::process::{Runner, find_program};

    let mut arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if arguments.first().is_some_and(|value| value == "--") {
        arguments.remove(0);
    }
    let Some(program) = arguments.first() else {
        return Err("Usage: wslc-session -- COMMAND [ARGUMENTS...]".into());
    };
    let program = find_program(program)?;
    let root = std::env::current_dir().map_err(|e| e.to_string())?;

    let seconds = std::env::var("WSLC_COMMAND_TIMEOUT")
        .unwrap_or_else(|_| "1800".into())
        .parse::<u64>()
        .map_err(|_| "WSLC_COMMAND_TIMEOUT must be positive")?;
    if seconds == 0 {
        return Err("WSLC_COMMAND_TIMEOUT must be positive".into());
    }

    with_connection(&root, &SessionOptions::from_env()?, |context| {
        let runner = Runner { root: &root };
        let mut command = runner.command(Path::new(&program), &arguments[1..]);
        context.configure_child(&mut command);

        Ok(runner
            .run_with_input(command, Duration::from_secs(seconds), false, true)?
            .code)
    })
}

#[cfg(not(windows))]
fn run() -> wslc_runtime::Result<i32> {
    Err("WSLC sessions require Windows".into())
}

fn main() -> ExitCode {
    if matches!(std::env::args().nth(1).as_deref(), Some("--help" | "-h")) {
        println!(
            "wslc-session -- COMMAND [ARGUMENTS...]\nRun a host command with WSLC_SESSION and WSLC_EXECUTABLE set.\nOwn and release a session unless WSLC_SESSION already names one.\nConfiguration: WSLC_CACHE, WSLC_STORAGE, WSLC_EXECUTABLE, WSLC_MEMORY_MB,\nWSLC_SDK_VERSION, WSLC_SDK_SHA256, WSLC_COMMAND_TIMEOUT (seconds)."
        );
        return ExitCode::SUCCESS;
    }
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("wslc-session {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    wslc_runtime::process::report(run())
}
