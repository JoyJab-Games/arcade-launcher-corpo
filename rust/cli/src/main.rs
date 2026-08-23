use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "arcade", about = "Admin CLI for the arcade cabinet launcher")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install a game: pick a source, resolve which game, choose a local
    /// reference name.
    Install,
    /// Re-provision an already-installed game and refresh its tags/preview
    /// image.
    Update { name: String },
    /// Remove an installed game.
    Remove { name: String },
    /// List installed games.
    List,
    /// Release a game to players (make it selectable in the front-end).
    Release { name: String },
    /// Unrelease a game (hide it from players again).
    Unrelease { name: String },
    /// Power off the device.
    Shutdown,
    /// Reboot the device.
    Reboot,
}

fn main() {
    let cli = Cli::parse();
    let store = arcade_core::GameStore::open(arcade_core::GameStore::default_root());

    let result = match cli.command {
        Command::Install => commands::install::run(&store),
        Command::Update { name } => commands::update::run(&store, &name),
        Command::Remove { name } => commands::remove::run(&store, &name),
        Command::List => commands::list::run(&store),
        Command::Release { name } => commands::release::run(&store, &name, true),
        Command::Unrelease { name } => commands::release::run(&store, &name, false),
        Command::Shutdown => commands::power::shutdown(),
        Command::Reboot => commands::power::reboot(),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
