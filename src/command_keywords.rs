pub(crate) static LOOKING_AROUND_COMMANDS: &[&str] = &[
    "ls", "ll", "la", "pwd", "cd", "tree", "find", "locate", "which", "whereis", "file", "stat",
    "du", "df", "dirname", "basename", "realpath", "readlink", "cat", "less", "more", "head",
    "tail", "grep", "rg", "fd", "eza", "exa", "bat", "nl", "tac", "wc", "mount", "lsblk", "blkid",
    "lsof", "ps", "top", "htop", "pgrep", "env", "printenv", "history", "alias", "type", "id",
    "uname", "hostname", "whoami",
];

pub(crate) static ROUTINE_COMMANDS: &[&str] = &[
    "mkdir", "touch", "cp", "mv", "echo", "clear", "sleep", "true", "false",
];

pub(crate) static WRAPPER_COMMANDS: &[&str] = &["sudo", "command", "builtin", "nohup", "time"];
pub(crate) static RUST_COMMANDS: &[&str] =
    &["cargo", "rustc", "rustfmt", "cargo-watch", "clippy-driver"];
pub(crate) static PYTHON_COMMANDS: &[&str] =
    &["python", "python3", "pip", "pip3", "pytest", "poetry"];
pub(crate) static PHP_COMMANDS: &[&str] = &["php", "composer", "artisan"];

pub(crate) static DESTRUCTIVE_COMMANDS: &[&str] = &[
    "rm", "rmdir", "kill", "pkill", "killall", "dd", "mkfs", "shutdown", "reboot",
];

pub(crate) static SERIOUS_COMMANDS: &[&str] = &[
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

pub(crate) static GIT_HAPPY_SUFFIXES: &[&str] =
    &["add", "commit", "push", "merge", "rebase", "tag", "stash"];
pub(crate) static GIT_SERIOUS_SUFFIXES: &[&str] = &[
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
pub(crate) static GIT_CURIOUS_SUFFIXES: &[&str] = &[
    "status", "diff", "log", "show", "blame", "branch", "remote", "grep",
];

pub(crate) static CARGO_HAPPY_SUFFIXES: &[&str] = &[
    "build", "check", "test", "run", "fmt", "clippy", "fix", "doc", "new",
];
pub(crate) static CARGO_SERIOUS_SUFFIXES: &[&str] = &[
    "update",
    "clean",
    "install",
    "uninstall",
    "publish",
    "vendor",
    "bench",
];
pub(crate) static CARGO_CURIOUS_SUFFIXES: &[&str] = &["tree", "metadata", "search"];

pub(crate) static PIP_ANGRY_SUFFIXES: &[&str] = &["install", "uninstall", "freeze", "list", "sync"];
pub(crate) static POETRY_ANGRY_SUFFIXES: &[&str] = &["install", "update", "add", "remove", "run"];

pub(crate) static COMPOSER_SAD_SUFFIXES: &[&str] = &[
    "install",
    "update",
    "dump-autoload",
    "require",
    "remove",
    "create-project",
];

pub(crate) static ARTISAN_SERIOUS_SUFFIXES: &[&str] =
    &["migrate", "queue:work", "schedule:run", "test"];
pub(crate) static ARTISAN_SAD_SUFFIXES: &[&str] = &[
    "serve",
    "cache:clear",
    "config:clear",
    "route:clear",
    "view:clear",
];

pub(crate) static TIME_QUENSTION_KEYWORDS: &[&str] = &["what", "time"];
