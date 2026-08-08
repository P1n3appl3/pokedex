use std::{ffi::OsStr, fs, io::Cursor, path::PathBuf, time::Duration};

use anyhow::{Result, anyhow};
use binrw::BinReaderExt;
use clap::{Args, Parser, Subcommand, ValueEnum};

use pokedex::{
    Game, Narc, Pokemon,
    fs::{Bytes, FsEntry, NitroFs},
    msg::decode_msg,
};
use rand::seq::IndexedRandom;

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
    Mon {
        name: String,

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

fn main() -> Result<()> {
    let args = Pokedex::parse();

    let rom = if let Some(game) = args.game { todo!() } else { random_rom()? };
    let rom = "diamond.nds";
    let rom = fs::read(rom)?;
    let fs = NitroFs::new(&rom)?;
    match args.command {
        Command::Mon { name, full, alt, shiny, verbosity, animate, period } => {
            pokedex_entry(
                &fs,
                name,
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
    Ok(())
}

fn pokedex_entry(
    fs: &NitroFs,
    name: String,
    full: bool,
    alt: bool,
    shiny: bool,
    verbosity: Verbosity,
    animate: Option<(Duration, Duration)>,
) -> Result<()> {
    let game = fs.header.game()?;
    let Some(FsEntry::File(contents)) = fs.get(game.text().into()) else { unreachable!() };
    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_le()?;
    let img_start = c.position() as u32 + narc.fnt_size;
    let msgs = game.msgs();
    macro_rules! msg {
        ($name:ident) => {{
            let temp = narc.fat[msgs.$name];
            let entry = &contents
                [(img_start + temp.start) as usize..(img_start + temp.end) as usize];
            decode_msg(entry)?
        }};
    }
    let names = msg!(name);

    let num = if name == "random" {
        rand::random_range(1..493)
    } else if let Ok(n) = name.parse::<usize>() {
        if !(1..493).contains(&n) {
            return Err(anyhow!("give a number between 1 and 493"));
        }
        n
    } else {
        names
            .iter()
            .position(|s| s == &name.to_uppercase())
            .expect("couldn't find that name")
    };
    println!("\n\n{} (#{num}) -- {}", names[num], msg!(nickname)[num]);
    println!("{} {}", msg!(height)[num], msg!(weight)[num]);
    println!("\n{}\n", msg!(pokedex_prose)[num]);

    let Some(FsEntry::File(contents)) = fs.get(game.stats().into()) else { unreachable!() };
    let mut c = Cursor::new(contents);
    let narc: Narc = c.read_le()?;
    let img_start = c.position() as u32 + narc.fnt_size;
    let temp = narc.fat[num];
    let entry =
        &contents[(img_start + temp.start) as usize..(img_start + temp.end) as usize];
    let mut c = Cursor::new(entry);
    let stats: Pokemon = c.read_le().unwrap();
    let types = msg!(r#type);
    if stats.type1 == stats.type2 {
        println!("{}/{} type", types[stats.type1 as usize], types[stats.type2 as usize]);
    } else {
        println!("{} type", types[stats.type1 as usize])
    }
    println!();

    println!("Hp    {}", stats.base_hp);
    println!("Atk   {}", stats.attack);
    println!("SpAtk {}", stats.special_attack);
    println!("Def   {}", stats.defence);
    println!("SpDef {}", stats.special_defence);
    println!("Speed {}", stats.speed);

    let Some(FsEntry::File(contents)) = fs.get(game.base_forms().into()) else {
        unreachable!()
    };
    let temp = u16::from_le_bytes(*contents.as_chunks::<2>().0[num].as_array().unwrap());
    println!("\nbase form is {} (#{temp})", names[temp as usize]);

    Ok(())
}

fn random_rom() -> Result<PathBuf> {
    let mut roms = Vec::new();
    for f in fs::read_dir(".")? {
        let f = f?;
        if f.path().is_file() && f.path().extension() == Some(OsStr::new("nds")) {
            roms.push(f.path().to_path_buf());
        }
    }
    roms.choose(&mut rand::rng()).cloned().ok_or_else(|| anyhow!("no roms found"))
}
