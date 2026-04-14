use clap::Parser;

use ccswitch::cli::{Cli, Commands};
use ccswitch::commands;
use ccswitch::config::{EXIT_KEYCHAIN, EXIT_NOT_FOUND, EXIT_USAGE};

fn main() {
    env_logger::init();

    // Internal daemon mode — launched by launchd with `--daemon` flag.
    // Checked before clap so it doesn't appear in --help.
    if std::env::args().any(|a| a == "--daemon") {
        ccswitch::daemon::run_daemon();
        return;
    }

    let cli = Cli::parse();
    let verbose = cli.verbose;

    let result = match &cli.command {
        Commands::List => commands::list::run(verbose),
        Commands::Status => commands::status::run(verbose),
        Commands::Add { name, overwrite } => commands::add::run(name, *overwrite, verbose),
        Commands::Use { name } => commands::use_profile::run(name, verbose),
        Commands::Remove { name, force } => commands::remove::run(name, *force, verbose),
        Commands::Daemon { command } => commands::daemon::run(command, verbose),
        Commands::Init => commands::init::run(verbose),
        Commands::Uninstall => commands::uninstall::run(verbose),
        Commands::Export { name, all, output, no_encrypt } => {
            commands::export::run(name.as_deref(), *all, output.as_deref(), *no_encrypt, verbose)
        }
        Commands::Import { file, r#as, overwrite } => {
            commands::import::run(file, r#as.as_deref(), *overwrite, verbose)
        }
    };

    if let Err(e) = result {
        let msg = e.to_string();
        ccswitch::output::print_error(&msg);

        let code = if msg.contains("keychain") || msg.contains("security CLI") {
            EXIT_KEYCHAIN
        } else if msg.contains("profile not found") || msg.contains("not found") {
            EXIT_NOT_FOUND
        } else {
            EXIT_USAGE
        };

        std::process::exit(code);
    }
}
