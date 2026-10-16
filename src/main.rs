use std::{
    ffi::OsStr,
    fmt::Write,
    fs,
    io::{self, Cursor},
    iter,
    path::PathBuf,
    time::Duration,
};

use anyhow::{Result, anyhow};
use binrw::BinReaderExt;
use clap::{Args, Parser, Subcommand, ValueEnum};
use owo_colors::{AnsiColors, OwoColorize, Rgb};
use rand::seq::IndexedRandom;

use pokedex::{
    Game, Narc, Pokemon, UPPER,
    fs::{Bytes, FsEntry, NitroFs},
    msg::decode_msg,
    sprite::{draw, get_palettes},
};

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
                &rom,
                name,
                Settings {
                    full,
                    alt,
                    shiny,
                    verbosity,
                    animate: animate.map(|t| {
                        (Duration::from_secs_f32(t), Duration::from_secs_f32(period))
                    }),
                },
            )
            .unwrap();
        }
        Command::Move {} => todo!(),
        Command::Ability {} => todo!(),
        Command::Search {} => todo!(),
    }
    Ok(())
}

struct Settings {
    full: bool,
    alt: bool,
    shiny: bool,
    verbosity: Verbosity,
    animate: Option<(Duration, Duration)>,
}

// n/255 = ?/width
const BLOCKS: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
fn stat(label: &str, value: u8, width: usize) -> String {
    assert_eq!(ansi_width::ansi_width(label), 3);
    let len = value as usize * width / 256;
    let remainder = (value as usize * 8 * width / 256) % 8;
    let color = match value {
        0..30 => Rgb(168, 0, 0),
        30..60 => Rgb(255, 132, 0),
        60..90 => Rgb(255, 214, 10),
        90..120 => Rgb(48, 148, 0),
        120..150 => Rgb(5, 255, 80),
        _ => Rgb(10, 240, 255),
    };
    format!(
        "{label} {value:3} {}{}{}",
        " ".repeat(len).on_color(color),
        BLOCKS[remainder].color(color),
        " ".repeat(width - len - 1)
    )
}

fn pokedex_entry(fs: &NitroFs, rom: &[u8], name: String, settings: Settings) -> Result<()> {
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
    let mut out = String::new();

    writeln!(&mut out, "\n\n{} (#{num})", names[num].bold())?;
    writeln!(&mut out, "{}", msg!(nickname)[num].bright_white().dimmed())?;
    let size =
        format!("{} / {}", msg!(height)[num].trim_start(), msg!(weight)[num].trim_start());
    writeln!(&mut out, "{}", size.dimmed())?;

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
    writeln!(&mut out)?;
    if stats.type1 != stats.type2 {
        writeln!(
            &mut out,
            "{}/{} type",
            types[stats.type1 as usize], types[stats.type2 as usize]
        )?
    } else {
        writeln!(&mut out, "{} type", types[stats.type1 as usize])?
    }
    let stat_width = 10;
    writeln!(&mut out, "\n{}\n", msg!(pokedex_prose)[num])?;
    writeln!(
        &mut out,
        "{} {}",
        stat("Hp ", stats.base_hp, stat_width),
        stat("Spe", stats.speed, stat_width)
    )?;
    writeln!(
        &mut out,
        "{} {}",
        stat("Atk", stats.attack, stat_width),
        stat("SpA", stats.special_attack, stat_width)
    )?;
    writeln!(
        &mut out,
        "{} {}",
        stat("Def", stats.defence, stat_width),
        stat("Spd", stats.special_defence, stat_width)
    )?;

    let Some(FsEntry::File(contents)) = fs.get(game.base_forms().into()) else {
        unreachable!()
    };
    let temp = u16::from_le_bytes(*contents.as_chunks::<2>().0[num].as_array().unwrap());
    writeln!(&mut out, "\nbase form is {} (#{temp})", names[temp as usize])?;

    let buf = &mut Vec::new();
    let palettes = get_palettes(rom, fs, buf);
    let main = draw(num, false, palettes, rom, fs)?;
    let alt = draw(num, true, palettes, rom, fs)?;

    let mut sprite = String::new();
    for [a, b] in main.as_chunks::<2>().0 {
        for (&top, &bottom) in a.iter().zip(b) {
            write!(
                &mut sprite,
                "{}",
                UPPER.color::<Rgb>(top.into()).on_color::<Rgb>(bottom.into())
            )
            .unwrap();
        }
        writeln!(&mut sprite).unwrap();
    }

    let width = termsize::get().map(|size| size.cols).unwrap();
    for (l, r) in iter::zip(out.lines(), sprite.lines().chain(iter::repeat(""))) {
        println!("{l}{}{r}", " ".repeat(width as usize / 2 - ansi_width::ansi_width(l)));
    }

    // for i in 0..=255 {
    //     println!("{}", stat("tst", i, 17));
    // }

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
