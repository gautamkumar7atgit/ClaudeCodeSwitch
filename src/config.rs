use std::path::PathBuf;

pub const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

/// The Keychain account name is the macOS login username, not a fixed string.
pub fn keychain_account() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| {
            std::process::Command::new("whoami")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| "root".to_string())
        })
}
pub const DAEMON_LOG_MAX_BYTES: u64 = 1_048_576; // 1 MB
pub const POLL_INTERVAL_SECS: u64 = 30;

// Exit codes
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_USAGE: i32 = 1;
pub const EXIT_KEYCHAIN: i32 = 2;
pub const EXIT_NOT_FOUND: i32 = 3;

pub struct Paths {
    pub profiles_dir: PathBuf,
    pub active_file: PathBuf,
    pub daemon_log: PathBuf,
    pub launch_agents_dir: PathBuf,
    pub plist_file: PathBuf,
}

pub fn paths() -> Paths {
    let home = dirs::home_dir().expect("cannot resolve home directory");
    let base = home.join(".claude-switcher");
    let launch_agents = home.join("Library").join("LaunchAgents");
    Paths {
        profiles_dir: base.join("profiles"),
        active_file: base.join("active"),
        daemon_log: base.join("daemon.log"),
        plist_file: launch_agents.join("com.ccswitch.daemon.plist"),
        launch_agents_dir: launch_agents,
    }
}
