use crate::Result;
use crate::syllables::SyllablePosition;
use crate::filesystemconfig::FileSystemConfig;
use std::collections::HashMap;
use std::fs;
use crate::error::LangErr;

pub trait LangConfig {
    fn syllables(&self) -> &Vec<String>;
    fn set_syllables(&mut self,syllables: Vec<String>);

    fn syllable_pos(&self) -> &HashMap<String,SyllablePosition>;
    fn set_syllable_pos(&mut self, syllable_pos: HashMap<String,SyllablePosition>);

    fn romanization(&self) -> &HashMap<String,String>;
    fn set_romanization(&mut self, utf_to_ascii: HashMap<String,String>);

    fn wanted(&self) -> &HashMap<String,f64>;
    fn set_wanted(&mut self, wanted: HashMap<String,f64>);

    fn database(&self) -> &Vec<String>;
    fn set_database(&mut self, db: Vec<String>);
    fn append_database(&mut self, words: Vec<String>);

    fn load(&mut self) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}
