type OsPath = String;

#[derive(Debug)]
pub enum LangErr {
    FileEmpty(OsPath),
    InvalidSyllable(String),
    InvalidSyllablePosition(String,usize),
    Io(std::io::Error),
    ParseIntError(std::num::ParseIntError),
}

impl From<std::io::Error> for LangErr {
    fn from(err: std::io::Error) -> Self {
        LangErr::Io(err)
    }
}

impl From<std::num::ParseIntError> for LangErr {
    fn from(err: std::num::ParseIntError) -> Self{ LangErr::ParseIntError(err)}
}