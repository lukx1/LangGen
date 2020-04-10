use crate::Result;
use crate::config::LangConfig;

pub trait RandomEngine {
    fn create_words(&mut self,min_len: u32, max_len: u32, count: u32, cfg: &dyn LangConfig) -> Vec<String>;
    fn with_config(cfg:&dyn LangConfig) -> Result<Self> where Self: Sized;
}