use clap::{Parser, Subcommand};
use plimeor_dotfiles::Dotfiles;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a dotfiles config file in the current directory
    Init,
    /// Collect dotfiles (link)
    Collect,
    /// Restore dotfiles (unlink)
    Restore {
        #[arg(short, long)]
        force: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init) => {
            Dotfiles::new();
        }
        Some(Commands::Collect) => {
            Dotfiles::collect().unwrap();
        }
        Some(Commands::Restore { force }) => {
            Dotfiles::restore(*force).unwrap();
        }
        None => {}
    }
}
