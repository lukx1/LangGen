#![feature(nll)]
#![feature(vec_remove_item)]

extern crate clap;
extern crate clipboard;
extern crate rand;

mod calculatedrandom;
mod config;
mod configcmd;
mod dbcmd;
mod error;
mod filesystemconfig;
mod gencmd;
mod rangen;
mod realrandom;
mod syllables;

use crate::config::LangConfig;
use crate::configcmd::ConfigCmd;
use crate::dbcmd::DatabaseCmd;
use crate::error::LangErr;
use crate::filesystemconfig::FileSystemConfig;
use crate::gencmd::GenerateCmd;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::borrow::BorrowMut;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, crate::error::LangErr>;

fn prepare_app<'a, 'b>() -> App<'a, 'b> {
    App::new("MyLang Generator")
        .version("0.3")
        .author("Lukx")
        .about("Generates stuff for my language")
        .subcommand(
            SubCommand::with_name("gen")
                .about("Generates a word or words")
                .arg(
                    Arg::with_name("count")
                        .short("c")
                        .long("count")
                        .help("Amount of words to generate")
                        .default_value("1")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("realrandom")
                        .long("realrandom")
                        .takes_value(false)
                        .help("Uses real RNG for all randomness"),
                )
                .arg(
                    Arg::with_name("min")
                        .short("m")
                        .long("min")
                        .default_value("1")
                        .takes_value(true)
                        .help("Minimum word length in syllables"),
                )
                .arg(
                    Arg::with_name("max")
                        .short("n")
                        .long("max")
                        .help("Maximum word length in syllables")
                        .default_value("4")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("length")
                        .short("l")
                        .long("length")
                        .help("Length of word(s)")
                        .takes_value(true)
                        .conflicts_with_all(&["min", "max"]),
                )
                .arg(
                    Arg::with_name("utf8")
                        .short("u")
                        .long("utf8")
                        .takes_value(false)
                        .help("Words will be displayed with UTF8 symbols"),
                )
                .arg(
                    Arg::with_name("clipboard")
                        .short("b")
                        .long("cb")
                        .help("Words will be saved to clipboard")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("db")
                        .short("db")
                        .help("Words will be added to the database")
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("db")
                .about("Word database manipulation")
                .arg(
                    Arg::with_name("add")
                        .short("add")
                        .long("add")
                        .help("Adds word or words to the database")
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("del")
                        .short("d")
                        .long("del")
                        .help("Deletes word or words from the database")
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("list")
                        .short("l")
                        .long("long")
                        .help("List all words in the database")
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about("Configuration")
                .arg(
                    Arg::with_name("wanted")
                        .short("w")
                        .long("wanted")
                        .takes_value(false)
                        .conflicts_with("real"),
                )
                .arg(
                    Arg::with_name("real")
                        .short("r")
                        .long("real")
                        .takes_value(false),
                ),
        )
}

/// Loads functions to be ran when they are called by the app
fn prepare_callers() -> TakeAppArgManager {
    let v: Vec<Box<dyn TakeAppArg>> = vec![
        Box::new(GenerateCmd::new()),
        Box::new(DatabaseCmd::new()),
        Box::new(ConfigCmd::new()),
    ];

    let mut subcommands = HashMap::new();

    v.into_iter().for_each(|c| {
        subcommands.insert(c.subcommand().to_string(), c);
    });

    TakeAppArgManager::new(subcommands)
}

/// Loads language config
fn prepare_lang_cfg() -> Box<dyn LangConfig> {
    let mut cfg = FileSystemConfig::default();
    cfg.load().unwrap();
    Box::new(cfg)
}

/// Called when err is encountered
fn handle_err(err: LangErr) {
    use crate::error::LangErr::*;

    match err {
        ParseIntError(e) => eprintln!("Integer could not be parsed: {:?}", e),
        InvalidSyllable(syllable) => eprintln!("Invalid syllable {}", syllable),
        Io(e) => eprintln!("Read or write error: {:?}", e),
        FileEmpty(e) => eprintln!("File {} is empty", e),
        InvalidSyllablePosition(syllable, pos) => {
            eprintln!("Syllable {} found in invalid position {}", syllable, pos)
        }
    }
}

fn are_launch_args_set() -> bool {
    std::env::args().len() > 1
}

fn main() {
    let lang_cfg = prepare_lang_cfg();

    let mut app = prepare_app();

    if !are_launch_args_set() {
        app.print_help().unwrap();
        return;
    } // Check if any args have been set

    let matches = app.get_matches();

    let mut callers = prepare_callers();

    let result = match matches.subcommand_name() {
        // Check what subcommand was set
        Some(sc) => callers
            .get(sc)
            .do_exec(matches.subcommand_matches(sc).unwrap(), lang_cfg),
        None => unreachable!(), // App will prevent unknown subcommands to reach this point
    };

    if let Err(e) = result {
        handle_err(e);
    }
}

/// Contains subcommands that can be ran by this app
struct TakeAppArgManager {
    /// (Subcommand -> TakeAppArg)
    subcommand_to_app_arg: HashMap<String, Box<dyn TakeAppArg>>,
}

impl TakeAppArgManager {
    pub fn new(data: HashMap<String, Box<dyn TakeAppArg>>) -> TakeAppArgManager {
        TakeAppArgManager {
            subcommand_to_app_arg: data,
        }
    }
    pub fn get(&mut self, subcommand: &str) -> &mut dyn TakeAppArg {
        self.subcommand_to_app_arg
            .get_mut(subcommand)
            .unwrap()
            .borrow_mut()
    }
}

/// Implement this for every subcommand and put the implementer into prepare_callers()
pub trait TakeAppArg {
    fn subcommand(&self) -> &str;
    fn do_exec(&mut self, arguments: &ArgMatches, cfg: Box<dyn LangConfig>) -> Result<()>;
}
