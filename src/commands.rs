use std::string::ToString;
use crate::TakeAppArg;
use crate::config::{LangConfig, InitableConfig};
use crate::Result;
use crate::error::LangErr;
use std::any::Any;
use crate::filesystemconfig::FileSystemConfig;

pub struct GenerateCmd {

}

impl GenerateCmd {
    pub fn new() -> GenerateCmd {
        unimplemented!()
    }
}

impl TakeAppArg for GenerateCmd {
    fn subcommand(&self) -> &str {
        unimplemented!()
    }

    fn name(&self) -> &str {
        unimplemented!()
    }

    fn call<T: Any + LangConfig>(&mut self, arguments: HashMap<String, Vec<String>, RandomState>, cfg: &T) -> Result<(), LangErr> {
        unimplemented!()
    }
}