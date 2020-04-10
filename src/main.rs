#![feature(nll)]

extern crate clap;
extern crate rand;
extern crate clipboard;

mod config;
mod error;
mod syllables;
mod filesystemconfig;
mod rangen;
mod calculatedrandom;
mod commands;

use clap::{App, Arg, SubCommand};
use crate::config::LangConfig;
use std::any::Any;
use std::collections::HashMap;
use crate::commands::GenerateCmd;

type Result<U> = std::result::Result<U,crate::error::LangErr>;

fn prepare_app<'a, 'b>() -> App<'a,'b>{
    App::new("MyLang Generator")
        .version("0.1")
        .author("Lukx")
        .about("Generates stuff for my language")
        .subcommand(SubCommand::with_name("gen")
            .about("Generates a word or words")
            .arg(Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Amount of words to generate")
                .default_value("1")
                .takes_value(true)
            )
            .arg(Arg::with_name("min")
                .short("m")
                .long("min")
                .default_value("1")
                .takes_value(true)
                .help("Minimum word length in syllables")
            )
            .arg(Arg::with_name("max")
                .short("n")
                .long("max")
                .help("Maximum word length in syllables")
                .default_value("4")
                .takes_value(true)
            )
            .arg(Arg::with_name("length")
                .short("l")
                .long("length")
                .help("Length of word(s)")
                .takes_value(true)
                .conflicts_with_all(&["min","max"])
            )
            .arg(Arg::with_name("utf8")
                .short("utf8")
                .takes_value(false)
                .help("Words will be displayed with UTF8 symbols")
            )
            .arg(Arg::with_name("clipboard")
                .short("b")
                .long("cb")
                .help("Words will be saved to clipboard")
                .takes_value(false)
            )
            .arg(Arg::with_name("db")
                .short("db")
                .help("Words will be added to the database")
                .takes_value(false)
            )
        )
        .subcommand(SubCommand::with_name("db")
            .about("Word database manipulation")
            .arg(Arg::with_name("add")
                .short("add")
                .long("add")
                .help("Adds word or words to the database")
                .takes_value(true)
                .multiple(true)
            )
            .arg(Arg::with_name("del")
                .short("d")
                .long("del")
                .help("Deletes word or words from the database")
                .takes_value(true)
                .multiple(true)
            )
            .arg(Arg::with_name("list")
                .short("l")
                .long("long")
                .help("List all words in the database")
                .takes_value(false)
            )
        )
        .subcommand(SubCommand::with_name("config")
            .about("Configuration")
            .arg(Arg::with_name("wanted")
                .short("w")
                .long("wanted")
                .takes_value(false)
                .conflicts_with("real")
            )
            .arg(Arg::with_name("real")
                .short("r")
                .long("real")
                .takes_value(false)
            )
        )
}

fn prepare_callers() -> HashMap<&str,Box<dyn TakeAppArg>> {
    let v:Vec<Box<dyn TakeAppArg>>  = vec![
        Box::new(GenerateCmd::new())
    ];

//    let mut result = HashMap::new(); TODO:tohle
//    v.iter()
//        .for_each(|c| result.insert(c.na))
}

fn main(){
    let app = prepare_app();
    let matches = app.get_matches();
    let callers = prepare_callers();

}

struct TakeAppArgManager {
    /// (Subcommand -> (Name ->) TakeAppArg)
    data: HashMap<String,HashMap<String,Box<dyn TakeAppArg>>>
}

impl TakeAppArgManager {
    pub fn get(&mut self,subcommand: &str, name: &str) -> Box<dyn TakeAppArg> {
        self.data.get(subcommand).unwrap()
            .remove(name).unwrap()
    }
}

pub trait TakeAppArg {
    fn subcommand(&self) -> &str;
    fn name(&self) -> &str;
    fn call<T: Any + LangConfig>(&mut self, arguments: HashMap<String,Vec<String>>, cfg: &T) -> Result<()>;
}