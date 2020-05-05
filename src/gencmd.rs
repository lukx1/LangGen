use crate::{TakeAppArg, rangen, syllables};
use crate::config::LangConfig;
use crate::Result;
use clap::ArgMatches;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::time::Duration;
use crate::rangen::RandomEngine;

pub const SUB_COMMAND: &'static str = "gen";

pub struct GenerateCmd;

impl GenerateCmd {
    pub fn new() -> GenerateCmd {
        GenerateCmd{}
    }
}

fn value_of_unsafe<'a>(matches : &'a ArgMatches, name: &str) -> &'a str {
    return matches.value_of(name).unwrap();
}

fn calc_min_max(matches: &ArgMatches) -> Result<(u32,u32)>{
    let (min,max);

    if matches.is_present("length") {
        min = value_of_unsafe(matches,"length").parse()?;
        max = value_of_unsafe(matches,"length").parse()?;
    }
    else {
        min = value_of_unsafe(matches,"min").parse()?;
        max = value_of_unsafe(matches,"max").parse()?;
    }

    Ok((min,max))
}

fn romanize(words: & Vec<String>, cfg: &dyn LangConfig) -> Vec<String>{
    words.iter()
        .map(|word| syllables::romanize(word,cfg).unwrap())
        .collect()
}

fn add_to_db(words: &Vec<String>, cfg: &mut dyn LangConfig){
    cfg.append_database(words);
}

fn copy_to_clipboard(words: &Vec<String>){
    let mut ctx :ClipboardContext = ClipboardProvider::new().expect("Clipboard provider failed");
    ctx.set_contents(words.join("\n")).expect("Clipboard set contents failed");
    std::thread::sleep(Duration::from_millis(500)); // This is a fix for some shitty bug in clipboard library -> https://github.com/aweinstock314/rust-clipboard/issues/61
}

fn print_words(words: &Vec<String>){
    words.iter()
        .for_each(|word| println!("{}",word));
}

fn choose_rangen(args: &ArgMatches,cfg: &dyn LangConfig) -> Box<dyn RandomEngine> {
    if args.is_present("realrandom"){
        return rangen::real_random(cfg);
    }
    else {
        rangen::calculated_random(cfg)
    }
}

impl TakeAppArg for GenerateCmd {
    fn subcommand(&self) -> &str {
        SUB_COMMAND
    }

    fn do_exec(&mut self, arguments: &ArgMatches,mut cfg: Box<dyn LangConfig>) -> Result<()> {
        let mut engine = choose_rangen(arguments,cfg.as_ref());

        let (min,max) = calc_min_max(arguments)?;
        let count = value_of_unsafe(arguments,"count").parse()?;

        let mut words = engine.create_words(min,max,count,cfg.as_ref());

        if arguments.is_present("db") {add_to_db(&words,cfg.as_mut());cfg.flush()?;}

        if arguments.is_present("romanize") {words = romanize(&words,cfg.as_ref());}

        print_words(&words); // Called before clipboard because clipboard may freeze

        if arguments.is_present("clipboard") {copy_to_clipboard(&words);}

        Ok(())
    }
}