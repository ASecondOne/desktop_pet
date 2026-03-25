use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

mod handle_event;

use handle_event::HookEvent;
use handle_event::PetResponce;

static DEBUG: bool = false;

struct SocketGuard {
    path: PathBuf,
}

impl Drop for SocketGuard {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!(
                "desktop_pet: failed to remove socket {}: {error}",
                self.path.display()
            );
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("desktop_pet: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let socket_path = parse_socket_path(env::args_os().skip(1))?;
    prepare_socket_path(&socket_path)?;
    let mut pending_events: HashMap<String, Vec<HookEvent>> = HashMap::new();

    let listener = UnixListener::bind(&socket_path)?;
    let _socket_guard = SocketGuard {
        path: socket_path.clone(),
    };

    print_startup(&socket_path)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_client(stream, &mut pending_events) {
                    eprintln!("desktop_pet: failed to read an event: {error}");
                }
            }
            Err(error) => eprintln!("desktop_pet: accept failed: {error}"),
        }
    }

    Ok(())
}

fn parse_socket_path<I>(mut args: I) -> Result<PathBuf, Box<dyn Error>>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    let mut socket_path = default_socket_path();

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--socket" => {
                let Some(path) = args.next() else {
                    return Err("missing value after --socket".into());
                };
                socket_path = PathBuf::from(path);
            }
            other => {
                return Err(format!("unknown argument: {other}").into());
            }
        }
    }

    Ok(socket_path)
}

fn default_socket_path() -> PathBuf {
    let username = env::var("USER").unwrap_or_else(|_| String::from("user"));
    env::temp_dir().join(format!("desktop_pet_{username}.sock"))
}

fn prepare_socket_path(path: &Path) -> Result<(), Box<dyn Error>> {
    let Some(parent) = path.parent() else {
        return Err("socket path must have a parent directory".into());
    };
    fs::create_dir_all(parent)?;

    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_socket() {
                match UnixStream::connect(path) {
                    Ok(_) => {
                        return Err(format!("socket already in use at {}", path.display()).into());
                    }
                    Err(_) => fs::remove_file(path)?,
                }
            } else {
                return Err(
                    format!("refusing to overwrite non-socket path {}", path.display()).into(),
                );
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

fn print_startup(socket_path: &Path) -> Result<(), Box<dyn Error>> {
    let hook_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("hooks")
        .join("desktop_pet_hook.zsh");

    println!("listening on {}", socket_path.display());
    println!("source this in every zsh terminal you want to mirror:");
    println!("  source {}", hook_path.display());
    println!(
        "default mode mirrors stdout/stderr for ordinary commands and leaves interactive ones alone"
    );
    println!("optional: export DESKTOP_PET_CAPTURE_OUTPUT=off for strict low-impact mode");
    println!(
        "optional: export DESKTOP_PET_CAPTURE_OUTPUT=always to mirror stdout/stderr for every command"
    );
    println!(
        "set DESKTOP_PET_SOCKET before sourcing; changing it later reroutes following commands"
    );

    if socket_path != default_socket_path() {
        println!("and set the socket before sourcing:");
        println!("  export DESKTOP_PET_SOCKET={}", socket_path.display());
    }

    println!("waiting for command events...");
    Ok(())
}

fn handle_client(
    stream: UnixStream,
    pending_events: &mut HashMap<String, Vec<HookEvent>>,
) -> Result<(), Box<dyn Error>> {
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<HookEvent>(&line) {
            Ok(event) => {
                if DEBUG {
                    HookEvent::print_event_debug(&event);
                }

                let command_id = event.command_id().to_owned();
                let is_finish = event.is_finish();

                pending_events
                    .entry(command_id.clone())
                    .or_default()
                    .push(event);

                if is_finish && let Some(events) = pending_events.remove(&command_id) {
                    let res = PetResponce::new(events);
                    println!("{}", res.show());
                }
            }
            Err(error) => eprintln!("desktop_pet: invalid event payload: {error}"),
        }
    }

    Ok(())
}
