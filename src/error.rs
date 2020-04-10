type OsPath = String;

#[derive(Debug)]
pub enum LangErr {
    FileEmpty(OsPath),
    InvalidSyllable(String),
    InvalidSyllablePosition(String,usize),
    Io(std::io::Error),
}

impl From<std::io::Error> for LangErr {
    fn from(io: std::io::Error) -> Self {
        LangErr::Io(io)
    }
}
