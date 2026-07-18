use std::time::Duration;

use anyhow::{Result, anyhow};
use clap::{Args, Parser, Subcommand, ValueEnum};

/// pokedex
#[derive(Parser, Debug)]
#[command(version, about)]
struct Pokedex {
    /// pick which rom to use
    #[arg(short, long)]
    game: Option<Game>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Pokemon {
        pokemon: String,

        /// show the full sprite (rather than box sprite)
        #[arg(short, long)]
        full: bool,

        /// show the second frame
        #[arg(short, long, conflicts_with = "animate")]
        alt: bool,

        /// show the shiny version of the sprite
        #[arg(short, long)]
        shiny: bool,

        #[command(flatten)]
        verbosity: Verbosity,

        /// animate the sprite for the given number of seconds
        #[arg(short, long)]
        animate: Option<f32>,

        /// how often to switch frames
        #[arg(short, long, requires = "animate", default_value_t = 0.25)]
        period: f32,
    },
    Move {},
    Ability {},
    Search {},
}

#[derive(Args, Debug, Clone)]
#[group(multiple = false)]
struct Verbosity {
    /// show a more limited pokedex entry
    #[arg(short, long)]
    short: bool,

    /// show a more verbose pokedex entry
    #[arg(short, long)]
    verbose: bool,
    /// only display certain components
    #[arg(short, long)]
    components: Vec<Component>,
}

#[derive(Clone, Debug, ValueEnum)]
enum Component {
    Name,
    Nickname,
    Number,
    Prose,
    Height,
    Weight,
    Sprite,
    Type,
    Effectiveness,
    Stats,
    Evolution,
    Abilities,
    Moves,
}

// move
// ability
// type
// search/list

fn main() {
    let args = Pokedex::parse();

    match args.command {
        Command::Pokemon { pokemon, full, alt, shiny, verbosity, animate, period } => {
            pokedex_entry(
                pokemon,
                full,
                alt,
                shiny,
                verbosity,
                animate
                    .map(|t| (Duration::from_secs_f32(t), Duration::from_secs_f32(period))),
            )
            .unwrap();
        }
        Command::Move {} => todo!(),
        Command::Ability {} => todo!(),
        Command::Search {} => todo!(),
    }
    println!("hello world")
}

fn pokedex_entry(
    mon: String,
    full: bool,
    alt: bool,
    shiny: bool,
    verbosity: Verbosity,
    animate: Option<(Duration, Duration)>,
) -> Result<()> {
    if mon == "random" {
        todo!("")
    }
    if let Ok(n) = mon.parse::<u32>() {
        if !(1..493).contains(&n) {
            return Err(anyhow!("give a number between 1 and 493"));
        }
        todo!()
    }
    todo!("");
    Ok(())
}
