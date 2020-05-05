use crate::Result;
use crate::config::LangConfig;
use crate::calculatedrandom::CalculatedRandom;
use crate::realrandom::RealRandom;

/// For structs that are able to generate words
pub trait RandomEngine {
    fn create_words(&mut self,min_len: u32, max_len: u32, count: u32, cfg: &dyn LangConfig) -> Vec<String>;
    fn with_config(cfg:&dyn LangConfig) -> Result<Self> where Self: Sized;
}

/// Purely random generation
pub fn real_random(cfg: &dyn LangConfig) -> Box<dyn RandomEngine>{
    Box::new(RealRandom::with_config(cfg).unwrap())
}

/// Deterministically chooses which syllables will move real occurrence closer to wanted occurrence
/// and then chooses from the best 10
pub fn calculated_random(cfg: &dyn LangConfig) -> Box<dyn RandomEngine>{
    Box::new(CalculatedRandom::with_config(cfg).unwrap())
}