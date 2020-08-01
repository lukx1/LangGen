use crate::config::LangConfig;
use crate::syllables::SyllablePosition;
use crate::Result;
use app_dirs::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const APP_INFO: AppInfo = AppInfo {
    name: "LangGen",
    author: "LukxNet",
};

const SYLLABLES_NAME: &str = "Syllables.txt";
const OCC_WANTED_NAME: &str = "Wanted.txt";
const SYLLABLE_VALID_POS_NAME: &str = "SyllablePos.txt";
const WORD_DATABASE_NAME: &str = "Word_Database.txt";
const SYLLABLES_TO_UTF8_NAME: &str = "SyllablesToUTF8.txt";

fn get_cfg_root() -> PathBuf {
    app_root(AppDataType::UserConfig, &APP_INFO).unwrap()
}

fn get_syllables_path() -> PathBuf {
    get_cfg_root().join(SYLLABLES_NAME)
}

fn get_occ_wanted_path() -> PathBuf {
    get_cfg_root().join(OCC_WANTED_NAME)
}

fn get_syllables_valid_pos_path() -> PathBuf {
    get_cfg_root().join(SYLLABLE_VALID_POS_NAME)
}

fn get_database_path() -> PathBuf {
    get_cfg_root().join(WORD_DATABASE_NAME)
}

fn get_syllables_to_utf8_path() -> PathBuf {
    get_cfg_root().join(SYLLABLES_TO_UTF8_NAME)
}

/// PathBuf to String, unsafe
fn pbts(p: PathBuf) -> String {
    p.canonicalize().unwrap().to_str().unwrap().to_string()
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        FileSystemConfig {
            syllables: Vec::new(),
            syllable_pos: HashMap::new(),
            utf8_to_ascii: HashMap::new(),
            wanted: HashMap::new(),
            database: Vec::new(),
            syllables_path: pbts(get_syllables_path()),
            wanted_path: pbts(get_occ_wanted_path()),
            syllable_pos_path: pbts(get_syllables_valid_pos_path()),
            database_path: pbts(get_database_path()),
            utf8_to_ascii_path: pbts(get_syllables_to_utf8_path()),
        }
    }
}

pub struct FileSystemConfig {
    // DATA
    syllables: Vec<String>,
    syllable_pos: HashMap<String, SyllablePosition>,
    utf8_to_ascii: HashMap<String, String>,
    wanted: HashMap<String, f64>,
    database: Vec<String>,
    // FILE PATHS
    syllables_path: String,
    syllable_pos_path: String,
    utf8_to_ascii_path: String,
    wanted_path: String,
    database_path: String,
}

impl FileSystemConfig {
    fn load_syllables(&mut self) -> Result<Vec<String>> {
        Ok(fs::read_to_string(&self.syllables_path)?
            .lines()
            .map(|s| s.to_string())
            .collect())
    }
    fn load_syllable_pos(&mut self) -> Result<HashMap<String, SyllablePosition>> {
        let mut result = HashMap::new();

        fs::read_to_string(&self.syllable_pos_path)?
            .lines()
            .for_each(|line| {
                let res: Vec<&str> = line.split(':').collect();
                let syllable = res[0].to_string();
                let pos = res[1].to_string().into();
                result.insert(syllable, pos);
            });

        Ok(result)
    }
    fn load_utf8_to_ascii(&mut self) -> Result<HashMap<String, String>> {
        Ok(fs::read_to_string(&self.utf8_to_ascii_path)?
            .lines()
            .map(parse_colon_separated_str_str)
            .collect())
    }
    fn load_wanted(&mut self) -> Result<HashMap<String, f64>> {
        Ok(fs::read_to_string(&self.wanted_path)?
            .lines()
            .map(parse_colon_separated_str_f64)
            .collect())
    }
    fn load_database(&mut self) -> Result<Vec<String>> {
        Ok(fs::read_to_string(&self.database_path)?
            .lines()
            .map(|s| s.to_string())
            .collect())
    }

    fn write_database(&mut self) -> Result<()> {
        let mut db = self.database.join("\n");
        db.push('\n');

        Ok(fs::write(&self.database_path, db)?)
    }
}

//TODO: Return results
fn parse_colon_separated_str_str(line: &str) -> (String, String) {
    let mut split = line.split(':');

    (
        split.next().unwrap().to_string(),
        split.next().unwrap().to_string(),
    )
}

//TODO: Return results
fn parse_colon_separated_str_f64(line: &str) -> (String, f64) {
    let mut split = line.split(':');

    (
        split.next().unwrap().to_string(),
        split.next().unwrap().to_string().parse().unwrap(),
    )
}

impl LangConfig for FileSystemConfig {
    fn syllables(&self) -> &Vec<String> {
        &self.syllables
    }

    fn set_syllables(&mut self, syllables: Vec<String>) {
        self.syllables = syllables;
    }

    fn syllable_pos(&self) -> &HashMap<String, SyllablePosition> {
        &self.syllable_pos
    }

    fn set_syllable_pos(&mut self, syllable_pos: HashMap<String, SyllablePosition>) {
        self.syllable_pos = syllable_pos
    }

    fn romanization(&self) -> &HashMap<String, String> {
        &self.utf8_to_ascii
    }

    fn set_romanization(&mut self, utf_to_ascii: HashMap<String, String>) {
        self.utf8_to_ascii = utf_to_ascii;
    }

    fn wanted(&self) -> &HashMap<String, f64> {
        &self.wanted
    }

    fn set_wanted(&mut self, wanted: HashMap<String, f64>) {
        self.wanted = wanted;
    }

    fn database(&self) -> &Vec<String> {
        &self.database
    }

    fn set_database(&mut self, db: Vec<String>) {
        self.database = db;
    }

    fn append_database(&mut self, words: &[String]) {
        words.iter().for_each(|w| self.database.push(w.to_string()));
    }

    fn delete_from_database(&mut self, word: &str) -> bool {
        self.database.remove_item(&word.to_string()).is_some()
    }

    fn load(&mut self) -> Result<()> {
        self.syllables = self.load_syllables()?;
        self.syllable_pos = self.load_syllable_pos()?;
        self.utf8_to_ascii = self.load_utf8_to_ascii()?;
        self.wanted = self.load_wanted()?;
        self.database = self.load_database()?;

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.write_database()
    }
}
