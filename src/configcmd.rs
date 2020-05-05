use crate::TakeAppArg;
use clap::ArgMatches;
use crate::config::LangConfig;
use crate::Result;
use crate::syllables;

pub struct ConfigCmd;

const SUBCOMMAND: &'static str = "config";

impl ConfigCmd {
    pub fn new() -> ConfigCmd {
        ConfigCmd
    }
}

fn wanted(_matches: &ArgMatches, cfg: &mut dyn LangConfig) -> Result<()> {
    let list = syllables::syllables_by_occourance_desc(cfg.wanted());

    for (syllable,percentage) in list {
        println!("{}: {:.2}%",&syllable,&percentage*100.0);
    }

    Ok(())
}


fn real(_matches: &ArgMatches, cfg: &mut dyn LangConfig) -> Result<()> {
    let list = syllables::syllables_by_occourance_desc(
        &syllables::db_syllable_occurrences_as_percentage(
            &syllables::db_syllable_occurrences_as_count(cfg).unwrap(),
            cfg)
            .unwrap()
    );

    for (syllable,percentage) in list {
        println!("{}: {:.2}%",&syllable,&percentage*100.0);
    }

    Ok(())
}


impl TakeAppArg for ConfigCmd {
    fn subcommand(&self) -> &str {
        SUBCOMMAND
    }

    fn do_exec(&mut self, arguments: &ArgMatches, mut cfg: Box<dyn LangConfig>) -> Result<()> {
        if arguments.is_present("wanted"){ wanted(arguments, cfg.as_mut())?;}
        if arguments.is_present("real"){ real(arguments, cfg.as_mut())?;}
        Ok(())
    }
}