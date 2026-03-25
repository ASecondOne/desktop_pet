use std::collections::HashMap;

use colored::Colorize;
use serde::Deserialize;

static LOOKING_AROUND_COMMANDS: &[&str] = &[
    "ls", "ll", "la", "pwd", "cd", "tree", "find", "locate", "which", "whereis", "file", "stat",
    "du", "df", "dirname", "basename", "realpath", "readlink", "cat", "less", "more", "head",
    "tail", "grep", "rg", "fd", "eza", "exa", "bat", "nl", "tac", "wc", "mount", "lsblk", "blkid",
    "lsof", "ps", "top", "htop", "pgrep", "env", "printenv", "history", "alias", "type", "id",
    "uname", "hostname", "whoami",
];
static ROUTINE_COMMANDS: &[&str] = &[
    "mkdir", "touch", "cp", "mv", "echo", "clear", "sleep", "true", "false",
];
static WRAPPER_COMMANDS: &[&str] = &["sudo", "command", "builtin", "nohup", "time"];
static RUST_COMMANDS: &[&str] = &["cargo", "rustc", "rustfmt", "cargo-watch", "clippy-driver"];
static PYTHON_COMMANDS: &[&str] = &["python", "python3", "pip", "pip3", "pytest", "poetry"];
static PHP_COMMANDS: &[&str] = &["php", "composer", "artisan"];
static DESTRUCTIVE_COMMANDS: &[&str] = &[
    "rm", "rmdir", "kill", "pkill", "killall", "dd", "mkfs", "shutdown", "reboot",
];
static SERIOUS_COMMANDS: &[&str] = &[
    "git",
    "docker",
    "docker-compose",
    "kubectl",
    "ssh",
    "scp",
    "rsync",
    "chmod",
    "chown",
    "systemctl",
    "service",
    "make",
    "cmake",
    "apt",
    "apt-get",
    "dnf",
    "pacman",
    "brew",
    "nix",
];
static GIT_HAPPY_SUFFIXES: &[&str] = &["add", "commit", "push", "merge", "rebase", "tag", "stash"];
static GIT_SERIOUS_SUFFIXES: &[&str] = &[
    "pull",
    "clone",
    "fetch",
    "checkout",
    "switch",
    "restore",
    "reset",
    "cherry-pick",
    "bisect",
];
static GIT_CURIOUS_SUFFIXES: &[&str] = &[
    "status", "diff", "log", "show", "blame", "branch", "remote", "grep",
];
static CARGO_HAPPY_SUFFIXES: &[&str] = &[
    "build", "check", "test", "run", "fmt", "clippy", "fix", "doc", "new",
];
static CARGO_SERIOUS_SUFFIXES: &[&str] = &[
    "update",
    "clean",
    "install",
    "uninstall",
    "publish",
    "vendor",
    "bench",
];
static CARGO_CURIOUS_SUFFIXES: &[&str] = &["tree", "metadata", "search"];
static PIP_ANGRY_SUFFIXES: &[&str] = &["install", "uninstall", "freeze", "list", "sync"];
static POETRY_ANGRY_SUFFIXES: &[&str] = &["install", "update", "add", "remove", "run"];
static COMPOSER_SAD_SUFFIXES: &[&str] = &[
    "install",
    "update",
    "dump-autoload",
    "require",
    "remove",
    "create-project",
];
static ARTISAN_SERIOUS_SUFFIXES: &[&str] = &["migrate", "queue:work", "schedule:run", "test"];
static ARTISAN_SAD_SUFFIXES: &[&str] = &[
    "serve",
    "cache:clear",
    "config:clear",
    "route:clear",
    "view:clear",
];

#[derive(Deserialize)]
pub struct HookEvent {
    timestamp: String,
    command_id: String,
    shell_pid: u32,
    tty: String,
    cwd: String,
    command: String,
    kind: HookKind,
    stream: Option<String>,
    text: Option<String>,
    exit_code: Option<i32>,
}

pub struct PetResponce {
    events: Vec<HookEvent>,
    emotion: Emotion,
    action: String,
    messge: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum HookKind {
    Start,
    Output,
    Finish,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
enum Emotion {
    HAPPY,
    SAD,
    ANGRY,
    SERIOUS,
    COURIOUS,
    NEUTRAL,
}

impl Emotion {
    pub fn get_all() -> [Emotion; 6] {
        [
            Emotion::HAPPY,
            Emotion::SAD,
            Emotion::ANGRY,
            Emotion::SERIOUS,
            Emotion::COURIOUS,
            Emotion::NEUTRAL,
        ]
    }
}

impl HookEvent {
    pub fn print_event_debug(event: &HookEvent) {
        let tty_label = event.tty.strip_prefix("/dev/").unwrap_or(&event.tty);

        match event.kind {
            HookKind::Start => {
                let timestamp = &event.timestamp;
                let shell_pid = event.shell_pid;
                let command = &event.command;
                let cwd = &event.cwd;

                println!("{}", "Start".green());

                println!(
                    "[{}] [{}:{}] $ {}    ({})",
                    timestamp, tty_label, shell_pid, command, cwd
                );

                println!();
            }
            HookKind::Output => {
                let stream = event.stream.as_deref().unwrap_or("stdout");
                let text = event.text.as_deref().unwrap_or("");

                let timestamp = &event.timestamp;
                let shell_pid = event.shell_pid;

                println!("{}", "OUTPUT".yellow());

                println!(
                    "[{}] [{}:{}:{}] {}",
                    timestamp, tty_label, shell_pid, stream, text
                );

                println!();
            }
            HookKind::Finish => {
                let exit_code = event.exit_code.unwrap_or_default();

                let timestamp = &event.timestamp;
                let shell_pid = event.shell_pid;
                let command = &event.command;

                println!("{}", "FINISH".red());

                println!(
                    "[{}] [{}:{}] exit {}    {}",
                    timestamp, tty_label, shell_pid, exit_code, command
                );

                println!();
            }
        }
    }

    pub fn is_finish(&self) -> bool {
        matches!(self.kind, HookKind::Finish)
    }

    pub fn command_id(&self) -> &str {
        &self.command_id
    }
}

fn bump_score(scores: &mut HashMap<Emotion, i32>, emotion: Emotion, amount: i32) {
    *scores.entry(emotion).or_insert(0) += amount;
}

fn command_parts(command: &str) -> Vec<&str> {
    command
        .split_whitespace()
        .map(|part| part.rsplit('/').next().unwrap_or(part))
        .collect()
}

fn effective_command_index(parts: &[&str]) -> Option<usize> {
    let mut index = 0;

    while index < parts.len() {
        if parts[index] == "env" {
            index += 1;
            while index < parts.len() && parts[index].contains('=') {
                index += 1;
            }
            continue;
        }

        if WRAPPER_COMMANDS.contains(&parts[index]) {
            index += 1;
            continue;
        }

        return Some(index);
    }

    None
}

impl PetResponce {
    pub fn new(events: Vec<HookEvent>) -> PetResponce {
        PetResponce {
            emotion: PetResponce::evaluate_emotion(&events),
            action: PetResponce::evaluate_action(&events),
            messge: PetResponce::evaluate_messege(&events),
            events,
        }
    }

    pub fn events(&self) -> &[HookEvent] {
        &self.events
    }

    pub fn show(&self) -> String {
        let _event_count = self.events().len();
        format!("{:?} {} \n {}", self.emotion, self.action, self.messge)
    }

    fn evaluate_emotion(events: &[HookEvent]) -> Emotion {
        let mut scores: HashMap<Emotion, i32> = HashMap::new();

        for e in Emotion::get_all() {
            scores.insert(e, 0);
        }

        for event in events {
            if let Some(ec) = event.exit_code {
                match ec {
                    0 => bump_score(&mut scores, Emotion::HAPPY, 1),
                    1 => bump_score(&mut scores, Emotion::SAD, 2),
                    130 | 137 | 143 => bump_score(&mut scores, Emotion::ANGRY, 2),
                    _ => {
                        bump_score(&mut scores, Emotion::ANGRY, 1);
                        bump_score(&mut scores, Emotion::SERIOUS, 1);
                    }
                }
            }

            let parts = command_parts(&event.command);
            let Some(command_index) = effective_command_index(&parts) else {
                continue;
            };

            let command = parts[command_index];
            let suffix = parts.get(command_index + 1).copied();
            let suffix2 = parts.get(command_index + 2).copied();

            if LOOKING_AROUND_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::COURIOUS, 2);
            } else if ROUTINE_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::NEUTRAL, 1);
            } else if DESTRUCTIVE_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::ANGRY, 3);
            } else if RUST_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::HAPPY, 2);
            } else if PYTHON_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::ANGRY, 3);
            } else if PHP_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::SAD, 3);
            } else if SERIOUS_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::SERIOUS, 2);
            }

            match command {
                "git" => match suffix {
                    Some(s) if GIT_HAPPY_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::HAPPY, 4);
                    }
                    Some(s) if GIT_SERIOUS_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::SERIOUS, 4);
                    }
                    Some(s) if GIT_CURIOUS_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::COURIOUS, 2);
                    }
                    Some(_) => bump_score(&mut scores, Emotion::SERIOUS, 1),
                    None => bump_score(&mut scores, Emotion::NEUTRAL, 1),
                },
                "cargo" => match suffix {
                    Some(s) if CARGO_HAPPY_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::HAPPY, 3);
                    }
                    Some(s) if CARGO_SERIOUS_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::SERIOUS, 2);
                    }
                    Some(s) if CARGO_CURIOUS_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::COURIOUS, 2);
                    }
                    Some(_) => bump_score(&mut scores, Emotion::SERIOUS, 1),
                    None => bump_score(&mut scores, Emotion::HAPPY, 1),
                },
                "python" | "python3" => {
                    bump_score(&mut scores, Emotion::ANGRY, 1);
                    if suffix == Some("-m") {
                        bump_score(&mut scores, Emotion::SERIOUS, 1);
                        if let Some(module) = suffix2 {
                            if module == "pip" || module == "pytest" {
                                bump_score(&mut scores, Emotion::ANGRY, 2);
                            }
                        }
                    }
                }
                "pip" | "pip3" => {
                    if matches!(suffix, Some(s) if PIP_ANGRY_SUFFIXES.contains(&s)) {
                        bump_score(&mut scores, Emotion::ANGRY, 2);
                    }
                }
                "poetry" => {
                    if matches!(suffix, Some(s) if POETRY_ANGRY_SUFFIXES.contains(&s)) {
                        bump_score(&mut scores, Emotion::ANGRY, 2);
                    } else {
                        bump_score(&mut scores, Emotion::SERIOUS, 1);
                    }
                }
                "composer" => {
                    if matches!(suffix, Some(s) if COMPOSER_SAD_SUFFIXES.contains(&s)) {
                        bump_score(&mut scores, Emotion::SAD, 2);
                    } else {
                        bump_score(&mut scores, Emotion::SERIOUS, 1);
                    }
                }
                "php" => {
                    if suffix == Some("artisan") {
                        bump_score(&mut scores, Emotion::SAD, 2);
                        if matches!(suffix2, Some(s) if ARTISAN_SERIOUS_SUFFIXES.contains(&s)) {
                            bump_score(&mut scores, Emotion::SERIOUS, 2);
                        }
                        if matches!(suffix2, Some(s) if ARTISAN_SAD_SUFFIXES.contains(&s)) {
                            bump_score(&mut scores, Emotion::SAD, 2);
                        }
                    }
                }
                "docker" | "docker-compose" | "kubectl" => {
                    bump_score(&mut scores, Emotion::SERIOUS, 2);
                }
                "ssh" | "scp" | "rsync" | "chmod" | "chown" | "systemctl" => {
                    bump_score(&mut scores, Emotion::SERIOUS, 1);
                }
                _ => {}
            }
        }

        let mut highest_score = 0;
        let mut highest_emotion = Emotion::NEUTRAL;
        for emotion in Emotion::get_all() {
            let score = *scores.get(&emotion).unwrap_or(&0);
            if score > highest_score {
                highest_score = score;
                highest_emotion = emotion;
            }
        }

        highest_emotion
    }

    fn evaluate_action(_events: &[HookEvent]) -> String {
        "WIP".to_string()
    }

    fn evaluate_messege(_events: &[HookEvent]) -> String {
        "HELLO".to_string()
    }
}
