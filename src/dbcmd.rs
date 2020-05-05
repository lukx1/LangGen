use crate::config::LangConfig;
use crate::Result;
use crate::TakeAppArg;
use clap::ArgMatches;

pub struct DatabaseCmd;

const SUBCOMMAND: &str = "db";

impl DatabaseCmd {
    pub fn new() -> DatabaseCmd {
        DatabaseCmd
    }
}

fn add(matches: &ArgMatches, cfg: &mut dyn LangConfig) -> Result<()> {
    let words = matches.values_of("add").unwrap();

    let mut wstr = Vec::with_capacity(words.len());

    for word in words {
        wstr.push(word.to_string());
    }

    cfg.append_database(&wstr);

    if wstr.len() > 1 {
        println!("Words were added to the database");
    } else {
        println!("Word was added to the database");
    }

    cfg.flush()
}

fn del(matches: &ArgMatches, cfg: &mut dyn LangConfig) -> Result<()> {
    let words = matches.values_of("del").unwrap();

    let mut successes = 0;
    let words_len = words.len(); // Can't iter words without moving

    for word in words {
        if cfg.delete_from_database(word) {
            successes += 1;
        }
    }

    if successes == words_len {
        println!("Words were deleted from the database");
    } else if successes == 0 {
        println!("Words were not found in the database");
    } else {
        println!("Some words could not be found and deleted from the database");
    }

    cfg.flush()?;
    Ok(())
}

fn list(_matches: &ArgMatches, cfg: &dyn LangConfig) -> Result<()> {
    println!("{}", cfg.database().join("\n"));

    Ok(())
}

impl TakeAppArg for DatabaseCmd {
    fn subcommand(&self) -> &str {
        SUBCOMMAND
    }

    fn do_exec(&mut self, arguments: &ArgMatches, mut cfg: Box<dyn LangConfig>) -> Result<()> {
        if arguments.is_present("add") {
            add(arguments, cfg.as_mut())?;
        } else if arguments.is_present("del") {
            del(arguments, cfg.as_mut())?;
        } else if arguments.is_present("list") {
            list(arguments, cfg.as_mut())?;
        } else {
            eprintln!("Invalid or no arguments have been specified");
        }

        Ok(())
    }
}
