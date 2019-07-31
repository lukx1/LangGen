#![feature(nll)]

extern crate clap;
extern crate rand;
extern crate clipboard;

use clap::{Arg, App, SubCommand, ArgMatches};
use std::collections::HashMap;
use std::{fs, error};
use core::borrow::{Borrow};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use std::error::Error;
use rand::Rng;
use rand::rngs::ThreadRng;
use core::fmt;
use std::path::Path;
use std::cmp::Ordering;
use std::process::exit;

/*const DIR_PATH: &str = "LangGen";
const SYLLABLES_PATH: &str = "LangGen/syllables.txt";
const WANTED_OCC: &str = "LangGen/wanted.txt";
const WORD_DATABASE: &str =  "LangGen/word_database.txt";
*/

const DIR_PATH:&str= "LangGenCfg";
const SYLLABLES_PATH:&str= "LangGenCfg/Syllables.txt";
const OCC_WANTED_PATH:&str= "LangGenCfg/Wanted.txt";
const SYLLABLE_VALID_POS_PATH:&str= "LangGenCfg/SyllablePos.txt";
const WORD_DATABASE_PATH:&str= "LangGenCfg/Word_Database.txt";
const SYLLABLES_TO_UTF8_PATH:&str= "LangGenCfg/SyllablesToUTF8.txt";

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

#[derive(PartialEq)]
enum SyllablePosition {
    START,
    END,
    MIDDLE,
    ANY,
}

enum DeleteResult {
    Ok,
    SomeNotFound,
    AllNotFound
}

#[derive(Debug,Clone)]
struct AlreadyInDBError;

impl fmt::Display for AlreadyInDBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Word is already in the database")
    }
}

impl Error for AlreadyInDBError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl From<String> for SyllablePosition {
    fn from(string:String) -> Self {
        match string.to_lowercase().borrow() {
            "start" => SyllablePosition::START,
            "end" => SyllablePosition::END,
            "middle" => SyllablePosition::MIDDLE,
            "any" => SyllablePosition::ANY,
            _ => panic!("Unexpected syllable position found in syllable_valid_pos_path")
        }
    }
}

struct MyLang {
    syllables: Vec<String>,
    syllable_valid_positions: HashMap<String,SyllablePosition>,
    occ_wanted: HashMap<String,f64>,
    occ_real: HashMap<String,f64>,
    config: Config,
}

struct Config {
    dir_path: String,
    syllables_path: String,
    occ_wanted_path: String,
    syllable_valid_pos_path: String,
    word_database_path: String,
    syllables_to_utf8_path:String,
}

impl Config {

    fn get_or_err(map: &HashMap<String,String>, name: &str) -> Result<String,std::io::Error> {
        if let Some(val) = map.get(name) {
            return Ok(val.to_string());
        }

        Err(std::io::Error::new(std::io::ErrorKind::NotFound,name))
    }

    pub fn create_file(&self,path:&str) -> Result<(),Box<Error>> {
        let mut output = String::new();

        output.push_str("dir_path:");
        output.push_str(&self.dir_path);
        output.push_str("\n");

        output.push_str("syllables_path:");
        output.push_str(&self.syllables_path);
        output.push_str("\n");

        output.push_str("occ_wanted_path:");
        output.push_str(&self.occ_wanted_path);
        output.push_str("\n");

        output.push_str("syllable_valid_pos_path:");
        output.push_str(&self.syllable_valid_pos_path);
        output.push_str("\n");

        output.push_str("word_database_path:");
        output.push_str(&self.word_database_path);
        output.push_str("\n");

        output.push_str("syllables_to_utf8_path:");
        output.push_str(&self.syllables_to_utf8_path);
        output.push_str("\n"); //TODO:Probably should not be here

        fs::write(path,output)?;

        Ok(())
    }

    pub fn get_invalid_files(&self) -> Vec<String>{
        let mut bad = Vec::new();

        if !Path::new(&self.dir_path).exists(){
            bad.push(self.dir_path.clone());
        }
        if !Path::new(&self.syllables_path).exists(){
            bad.push(self.syllables_path.clone());
        }
        if !Path::new(&self.occ_wanted_path).exists(){
            bad.push(self.occ_wanted_path.clone());
        }
        if !Path::new(&self.syllable_valid_pos_path).exists(){
            bad.push(self.syllable_valid_pos_path.clone());
        }
        if !Path::new(&self.word_database_path).exists(){
            bad.push(self.word_database_path.clone());
        }
        if !Path::new(&self.syllables_to_utf8_path).exists(){
            bad.push(self.syllables_to_utf8_path.clone());
        }

        bad
    }

    pub fn new(path:&str) -> Result<Config,Box<Error>> {
        let mut map = HashMap::new();

        for line in fs::read_to_string(path)?.lines(){
            let parts:Vec<&str> = line.split(":").collect();

            map.insert(parts[0].to_string(),parts[1].to_string());
        }

        Ok(Config {
            dir_path: Config::get_or_err(&map,"dir_path")?.to_string(),
            syllables_path: Config::get_or_err(&map,"syllables_path")?.to_string(),
            occ_wanted_path: Config::get_or_err(&map,"occ_wanted_path")?.to_string(),
            syllable_valid_pos_path: Config::get_or_err(&map,"syllable_valid_pos_path")?.to_string(),
            word_database_path: Config::get_or_err(&map,"word_database_path")?.to_string(),
            syllables_to_utf8_path: Config::get_or_err(&map,"syllables_to_utf8_path")?.to_string(),
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dir_path: DIR_PATH.to_string(),
            syllables_path: SYLLABLES_PATH.to_string(),
            occ_wanted_path: OCC_WANTED_PATH.to_string(),
            syllable_valid_pos_path: SYLLABLE_VALID_POS_PATH.to_string(),
            word_database_path: WORD_DATABASE_PATH.to_string(),
            syllables_to_utf8_path: SYLLABLES_TO_UTF8_PATH.to_string()
        }
    }
}

impl MyLang {
    pub fn new() -> MyLang {
        MyLang{
            syllables: Vec::new(),
            syllable_valid_positions: HashMap::new(),
            occ_wanted: HashMap::new(),
            occ_real: HashMap::new(),
            config: Config::default()
        }
    }

    pub fn new_with_config(config: Config) -> MyLang {
        MyLang{
            syllables: Vec::new(),
            syllable_valid_positions: HashMap::new(),
            occ_wanted: HashMap::new(),
            occ_real: HashMap::new(),
            config
        }
    }

    pub fn is_syllable_valid_at_pos(&self, syllable:&str, pos:u8, len:u8) -> bool {
        let len = len-1; // length to index of last syllable
        if let Some(sp) = self.syllable_valid_positions.get(syllable) {
            match(sp,pos) {
                (SyllablePosition::START,0) => return true,
                (SyllablePosition::START,_) => return false,
                (SyllablePosition::MIDDLE,0) => return false,
                (SyllablePosition::MIDDLE,len) => return false,
                (SyllablePosition::MIDDLE,_) => return true,
                (SyllablePosition::END,len) => return true,
                (SyllablePosition::ANY,_) => return true,
            }
        }
        else {
            return false;
        }
    }

    pub fn convert_to_utf8(&self, word:&str) -> Result<String,Box<Error>> {
        let syllables = self.split_word_into_syllables(word)?;

        let mut map = HashMap::new();

        for line in BufReader::new(File::open(&self.config.syllables_to_utf8_path)?).lines(){
            let uw = line?;
            let res: Vec<&str> = uw.split(":").collect();

            let normal = res[0].to_string();
            let utf = res[1].to_string();

            map.insert(normal,utf);
        }

        let mut string = String::new();

        syllables.iter().for_each(|s| string += map.get(s).unwrap());//TODO:Unsafe

        Ok(string)
    }

    fn is_word_valid(&self, word:&str) -> bool {
        if word.trim() == "" {return false;}

        let syllables = match self.split_word_into_syllables(word) {
            Ok(s) => s,
            _ => return false
        };

        for (pos,syllable) in syllables.iter().enumerate() {
            if !self.is_syllable_valid_at_pos(syllable,pos as u8,syllables.len() as u8) {
                return false;
            }
        }

        return true;
    }

    fn split_word_into_syllables(&self,word:&str) -> Result<Vec<String>,std::fmt::Error> {
        let mut w = word.trim();

        let mut result = vec![];

        let mut pos = 0u16;
        let mut buffer = String::new();
        for c in w.chars() {
            buffer.push(c);
            if self.syllables.contains(&buffer) {
                result.push(buffer.clone());
                buffer.clear();
            }

            pos += 1;
        }

        if buffer.len() > 0{
            if self.syllables.contains(&buffer) {
                result.push(buffer);
            }
            else {
                return Err(std::fmt::Error);
            }
        }

        Ok(result)
    }

    fn get_real_occ_as_count(&self) -> Result<HashMap<String,u32>,Box<Error>> {
        let mut occs_count: HashMap<String,u32> = HashMap::new();

        for line in BufReader::new(File::open(&self.config.word_database_path)?).lines(){ // Count all syllables
            let syllables = self.split_word_into_syllables(&line?)?;
            syllables.iter().for_each(|s| {
                if occs_count.contains_key(s) {
                    occs_count.insert(s.clone(),occs_count.get(s).unwrap()+1);
                }
                else {
                    occs_count.insert(s.clone(),1);
                }
            })
        }

        if self.syllables.len() == 0 {
            return Err(Box::new(std::fmt::Error)); //TODO:Change this error to something better
        }

        for syllable in &self.syllables { // Add syllables that werent found in word_db
            if !occs_count.contains_key(syllable) {
                occs_count.insert(syllable.clone(),0);
            }
        }

        Ok(occs_count)
    }

    fn get_real_occ_as_perc_have_count(&self, occs_count: &HashMap<String,u32>) -> HashMap<String,f64>{
        let mut total = occs_count.iter().map(|(_,c)| *c).sum::<u32>() as f64; // Make sum of all syllables

        let mut result = HashMap::new();

        for (s,c) in occs_count.borrow(){
            result.insert(s.clone(),*c as f64 / total); // Convert to number of syllables to percentage
        }

        if result.len() < self.syllables.len() { // Not every syllable is in word db
            for syllable in &self.syllables { // Check every syllable
                if !result.contains_key(syllable) { // If it isn't added, then add it
                    result.insert(syllable.clone(),0.0);
                }
            }
        }

        result
    }

    fn get_real_occ_as_perc(&self) -> Result<HashMap<String,f64>,Box<Error>> {
        let occs_count = self.get_real_occ_as_count();

        Ok(self.get_real_occ_as_perc_have_count(&occs_count?))
    }

    fn get_syllable_valid_positions(path:&str) -> HashMap<String,SyllablePosition> {
        let mut map = HashMap::new();

        fs::read_to_string(path).unwrap().lines().for_each(|line| {
            let res: Vec<&str> = line.split(":").collect();
            let syllable = res[0].to_string();
            let pos = res[1].to_string().into();
            map.insert(syllable,pos);
        });

        map
    }

    fn get_syllables(path:&str) -> Vec<String> {
        let mut syllables = Vec::new();

        fs::read_to_string(path).unwrap().lines().for_each(|s| syllables.push(s.to_string()));

        syllables.sort_by(|a,b| a.len().cmp(&b.len()));

        syllables
    }

    fn does_db_contain(&self, word: &str) -> Result<bool,Box<Error>> {
        for line in BufReader::new(File::open(&self.config.word_database_path)?).lines(){
            if line? == word {
                return Ok(true);
            }
        }
        return Ok(false);
    }

    pub fn recalc_real(&mut self){
        self.occ_real = self.get_real_occ_as_perc().unwrap();
    }

    pub fn add_to_db(&mut self,word: &str) -> Result<(),Box<Error>>{
        if !self.is_word_valid(word) {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound,word)));
        }

        if self.does_db_contain(word)?{
            return Err(Box::new(AlreadyInDBError));
        }

        let mut file =OpenOptions::new().append(true).open(&self.config.word_database_path)?;
        file.write_all(format!("{}\n",word).as_bytes())?;

        Ok(())
    }

    pub fn list_db(&self) -> Result<Vec<String>,Box<Error>> {
        let mut vector = Vec::new();
        for line in BufReader::new(File::open(&self.config.word_database_path)?).lines(){
            vector.push(line?);
        }

        Ok(vector)
    }

    pub fn delete_from_db(&self, words: &Vec<String>) -> Result<DeleteResult,Box<Error>>{
        let mut copy = Vec::new();

        let mut words_deleted = 0usize;

        for line in BufReader::new(File::open(&self.config.word_database_path)?).lines(){
            let uw = line?;
            if words.contains(&(uw.clone())) {
                words_deleted += 1;
                continue;
            }

            copy.push(uw);
        }

        let mut file = OpenOptions::new().write(true).open(&self.config.word_database_path)?;
        for line in copy {
            file.write(line.as_bytes())?;
            file.write("\n".as_bytes())?;
        }

        if words_deleted == words.len() { return Ok(DeleteResult::Ok);}
        if words_deleted > 0 { return Ok(DeleteResult::SomeNotFound);}
        Ok(DeleteResult::AllNotFound)
    }

    fn get_syllable_wanted(path: &str) -> HashMap<String,f64> {
        let mut map:HashMap<String,f64> = HashMap::new();

        fs::read_to_string(path).unwrap().lines().for_each(|line| {
            let res: Vec<&str> = line.split(":").collect();
            map.insert(res[0].to_string(), res[1].parse::<f64>().unwrap());
        });

        map
    }

    pub fn init(&mut self){
        self.syllables = MyLang::get_syllables(&self.config.syllables_path);
        self.occ_wanted = MyLang::get_syllable_wanted(&self.config.occ_wanted_path);
        self.occ_real = self.get_real_occ_as_perc().unwrap();//TODO:Unsafe
        self.syllable_valid_positions = MyLang::get_syllable_valid_positions(&self.config.syllable_valid_pos_path);
    }

    pub fn replace_config(&mut self, config:Config){
        self.config = config;
    }
}

enum RandomnessMethod {
    Pure,
    Linear,
    Calculated,
}

trait RandomGen {
    fn create_words(&mut self, lang: &MyLang,min:u8,max:u8,count:u8) -> Vec<String>;
    fn recalculate(&mut self, lang: &MyLang);
    fn invalidate_cache(&mut self, lang: &MyLang);
}

struct CalculatedRandom {
    occ_count: HashMap<String,u32>,
    temp_map: HashMap<String,f64>,
    sum: u32,
    cache_ready: bool,
}

fn comp_f64(a: &f64, b: &f64) -> Ordering {
    if a < b {
        return Ordering::Less;
    } else if a > b {
        return Ordering::Greater;
    }
    Ordering::Equal
}

impl CalculatedRandom {
    fn new() -> CalculatedRandom {
        CalculatedRandom{
            occ_count: HashMap::new(),
            sum:0,
            temp_map: HashMap::new(),
            cache_ready: false,
        }
    }

    fn order_temp_map(&mut self) -> Vec<String> {
        let mut v:Vec<String> = Vec::with_capacity(self.temp_map.len());

        self.temp_map.iter().for_each(|(s,ob)| v.push(s.clone()));

        v.sort_by(|a,b| comp_f64(self.temp_map.get(a).unwrap(),self.temp_map.get(b).unwrap()));

        v
    }

    fn pull_syllable(&mut self, lang: &MyLang, pos:u8, len:u8) -> String{
        if !self.cache_ready {
            self.init_cache(lang);
        }

        self.temp_map.clear();

        //How would offby change if we pulled this syllable
        for s_po in &lang.syllables { // List through syllable that may be pulled
            let mut offby = 0.0; // How much values are off wanted values
            for (s_o,count) in &self.occ_count { // Compare syllable pulled s_po to others s_o
                let mut cadj= *count;
                if s_o == s_po { // If this syllable was pulled
                    cadj += 1; // Add one
                }
                //Otherwise it stays the same

                let adj_real = cadj as f64 / self.sum as f64; // Calculate frequency

                offby += (*lang.occ_wanted.get(s_o).unwrap() - (adj_real)).abs(); // Calculate offby for this syllable
            }
            self.temp_map.insert(s_po.clone(),offby); // Add this option to sorted map
        }


//        for s_po in &lang.syllables {
//            let freq_after = (*self.occ_count.get(s_po).unwrap() as f64 + 1.0) / (self.sum as f64 +1.0);
//
//            self.temp_map.insert(s_po.clone(),freq_after);

        for (s,c) in &self.temp_map {
            println!("{}:{}",s,c);
        }

        let mut possible_results = Vec::with_capacity(self.order_temp_map().len()/10);
        let mut i = 0;

        for s in self.order_temp_map(){ // Iterate the sorted map
            if lang.is_syllable_valid_at_pos(&s,pos,len) {//Until you find a valid char
                possible_results.push(s);

                i+=1;

                if i == possible_results.capacity()-1 {
                    break;
                }
            }
        }

        let syllable_result = possible_results[rand::thread_rng().gen_range(0,possible_results.len())].clone();

        if syllable_result == "" { // If no valid char has been found
            soft_crash(format!("No suitable syllable could be pulled from DB\n 0 out of {} valid",lang.syllables.len())); // Stop
        }

        // Safe unwrap
        // Increase occurance of syllable just pulled
        self.occ_count.insert(syllable_result.clone(),self.occ_count.get(&syllable_result).unwrap()+1);

        // Increase sum
        self.sum += 1;

        syllable_result
    }

    fn init_cache(&mut self, lang: &MyLang){
        self.occ_count = lang.get_real_occ_as_count().unwrap();
        self.sum = self.occ_count.iter().map(|(_,c)| *c).sum::<u32>(); // Make sum of all syllables
        self.cache_ready = true;
        self.temp_map =  HashMap::new();
    }
}

impl RandomGen for CalculatedRandom {
    fn create_words(&mut self, lang: &MyLang, min: u8, max: u8, count: u8) -> Vec<String> {
        let mut result = Vec::with_capacity(count as usize);

        let mut rng = rand::thread_rng();
        for nth_word in 0..count {

            let chosen_length = rng.gen_range(min,max+1);
            let mut word = String::new();
            let mut pos = 0u8;

            for nth_syllable in 0..chosen_length {
                word.push_str(&self.pull_syllable(lang,pos,chosen_length));
                pos += 1;
            }

            result.push(word);
        }

        result
    }

    fn recalculate(&mut self, lang: &MyLang) {
        /* Recalc is done by pull_syllable */
    }

    fn invalidate_cache(&mut self, lang: &MyLang) {
        self.init_cache(lang);
    }
}

struct RandomEngine {
    my_lang:MyLang,
    rng: ThreadRng,
    random_gen: Box<RandomGen>,
}

impl RandomEngine {
    fn new(mut my_lang: MyLang, random_gen : Box<RandomGen>) -> RandomEngine {
        if my_lang.syllables.len() < 1 {
            my_lang.init();
        }

        RandomEngine {
            my_lang,
            rng: rand::thread_rng(),
            random_gen
        }
    }

//    fn get_or_zero(&self,syllable:&str, map: &HashMap<String,f64>) -> f64 {
//         if let Some(occ) = map.get(syllable) {
//             return occ.clone();
//         }
//
//        0.0
//    }

//    fn is_word_db_empty(&self) -> bool {
//        return self.my_lang.occ_real.iter().all(|(s,o)| *o == 0.0);
//    }

    /*fn calc_adjusted_random_calculated(&mut self) -> HashMap<String,f64> {

    }*/


//    fn calc_adjusted_random_linear(&mut self) -> HashMap<String,f64> {
//        if self.is_word_db_empty() { //Everything is zero, nothing can be adjusted
//            return self.my_lang.occ_wanted.clone();
//        }
//
//        let mut adjusted = HashMap::new();
//
//        for syllable in &self.my_lang.syllables {
//            let wanted = self.get_or_zero(syllable,&self.my_lang.occ_wanted);
//            let real = self.get_or_zero(syllable,&self.my_lang.occ_real);
//
//            let adj:f64 = wanted + wanted * (1.0 - real/wanted);
//
//            adjusted.insert(syllable.clone(),adj);
//        }
//
//        adjusted
//    }

    pub fn recalc_adjusted_random(&mut self){
        self.random_gen.recalculate(&self.my_lang);
    }

//    fn pull_random_syllable(&mut self, pos:u8, len:u8, attempts_made:u8) -> String {
//        let rnd:f64 = self.rng.gen();
//
//        let mut result = String::new();
//
//        let mut lower = 0.0;
//
//
//        for (syllable,occ) in &self.occ_adjusted {
//            let upper = lower+occ;
//
//            if rnd >= lower && rnd < upper{
//                result = syllable.clone();
//                break;
//            }
//
//            lower += occ;
//        }
//
//        if !self.my_lang.is_syllable_valid_at_pos(&result,pos,len) {
//            if attempts_made > 100 {panic!("No suitable syllable could be made");}
//            return self.pull_random_syllable(pos,len,attempts_made+1);
//        }
//
//        if result != "" {
//            return result;
//        }
//        panic!("No suitable random syllable was found (is syllable list empty?)")
//    }

    pub fn create_words(&mut self,min:u8,max:u8,count:u8) -> Vec<String> {
        self.random_gen.create_words(&self.my_lang,min,max,count)
    }

}

fn prepare_app<'a, 'b>() -> App<'a,'b>{
    App::new("MyLang Generator")
        .version("0.1")
        .author("Lukx")
        .about("Generates stuff for my language")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .help("Path to base config")
            .takes_value(true)
            .multiple(false)
            .default_value("Config.cfg")
        )
        .arg(Arg::with_name("init_config")
            .short("i")
            .long("iconf")
            .help("Initialize files and directories based on config")
            .takes_value(false)
        )
        .subcommand(SubCommand::with_name("gen")
            .about("Generates a word or words")
            .arg(Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Amount of words to generate")
                .default_value("1")
                .takes_value(true)
            )
            .arg(Arg::with_name("min")
                .short("m")
                .long("min")
                .default_value("1")
                .takes_value(true)
                .help("Minimum word length in syllables")
            )
            .arg(Arg::with_name("max")
                .short("n")
                .long("max")
                .help("Maximum word length in syllables")
                .default_value("4")
                .takes_value(true)
            )
            .arg(Arg::with_name("length")
                .short("l")
                .long("length")
                .help("Length of word(s)")
                .takes_value(true)
                .conflicts_with_all(&["min","max"])
            )
            .arg(Arg::with_name("utf8")
                .short("utf8")
                .takes_value(false)
                .help("Words will be displayed with UTF8 symbols")
            )
            .arg(Arg::with_name("clipboard")
                .short("b")
                .long("cb")
                .help("Words will be saved to clipboard")
                .takes_value(false)
            )
            .arg(Arg::with_name("db")
                .short("db")
                .help("Words will be added to the database")
                .takes_value(false)
            )
        )
        .subcommand(SubCommand::with_name("db")
            .about("Word database manipulation")
            .arg(Arg::with_name("add")
                .short("add")
                .long("add")
                .help("Adds word or words to the database")
                .takes_value(true)
                .multiple(true)
            )
            .arg(Arg::with_name("del")
                .short("d")
                .long("del")
                .help("Deletes word or words from the database")
                .takes_value(true)
                .multiple(true)
            )
            .arg(Arg::with_name("list")
                .short("l")
                .long("long")
                .help("List all words in the database")
                .takes_value(false)
            )
        )
        .subcommand(SubCommand::with_name("config")
            .about("Configuration")
            .arg(Arg::with_name("wanted")
                .short("w")
                .long("wanted")
                .takes_value(false)
                .conflicts_with("real")
            )
            .arg(Arg::with_name("real")
                .short("r")
                .long("real")
                .takes_value(false)
            )
        )
}

fn handle_db(matches : &ArgMatches, my_lang:&mut MyLang) -> Result<(),Box<Error>>{
    if matches.is_present("add"){
        let words = matches.values_of("add").unwrap();

        for val in words {
            my_lang.add_to_db(val)?;
        }
    }
    else if matches.is_present("del"){
        let words = matches.values_of("del").unwrap();

        let mut vec = Vec::new();

        for val in words {
            vec.push(val.to_string());
        }

        match my_lang.delete_from_db(&vec)? {
            DeleteResult::Ok => println!("Deleted from database"),
            DeleteResult::SomeNotFound => println!("Some words were not found in DB and could not be delted"),
            DeleteResult::AllNotFound => println!("Not found in DB")
        }
    }
    else if matches.is_present("list"){
        for word in my_lang.list_db()?{
            println!("{}",word);
        }
    }
    else {
        eprintln!("Parameter must be specified (-add -del -list)");
    }
    Ok(())
}


fn value_of_unsafe<'a>(matches : &'a ArgMatches, name: &str) -> &'a str {
    return matches.value_of(name).unwrap();
}

fn vec_to_str(vec:&Vec<String>) -> String{
    let mut string = String::new();

    for (i,s) in vec.iter().enumerate(){
        if i != 0 {
            string += LINE_ENDING;
        }

        string += &s.clone();
    };

    string
}

fn validate_config(my_lang:&MyLang) -> bool {
    let mut valid = true;

    for file in my_lang.config.get_invalid_files(){
        if !file.contains(".") {
            eprintln!("Directory {} not found",file);
        }
        else {
            eprintln!("File {} not found",file);
        }

        valid = false;
    }

    valid
}

fn handle_gen(matches : &ArgMatches, mut random_engine: RandomEngine) -> Result<(), Box<Error>>{
    use clipboard::{ClipboardProvider, ClipboardContext};

    random_engine.recalc_adjusted_random();

    let mut min;
    let mut max;

    if matches.is_present("length") {
        min = value_of_unsafe(matches,"length").parse()?;
        max = value_of_unsafe(matches,"length").parse()?;
    }
    else {
        min = value_of_unsafe(matches,"min").parse()?;
        max = value_of_unsafe(matches,"max").parse()?;
    }

    let count : u8 = value_of_unsafe(matches,"count").parse()?;

    let add_to_db = matches.is_present("db");

    let words = random_engine.create_words(min,max,count);

    for word in &words {
        if matches.is_present("utf8") {
            println!("{}",random_engine.my_lang.convert_to_utf8(&word)?);
        }
        else {
            println!("{}",word);
        }

        if add_to_db {
            random_engine.my_lang.add_to_db(word)?;
        }
    }

    if matches.is_present("clipboard") {
        let mut ctx :ClipboardContext = ClipboardProvider::new()?;
        ctx.set_contents(vec_to_str(&words))?;
    }



    Ok(())
}

fn handle_config(matches : &ArgMatches, my_lang:&mut MyLang){
    if matches.is_present("wanted"){
        for (s,o) in &my_lang.occ_wanted {
            println!("{}:{:.1}",s,o);
        }
    }
    else if matches.is_present("real") {
        for (s,o) in &my_lang.occ_real {
            println!("{}:{:.1}",s,o);
        }
    }
    else {
        eprintln!("Parameter must be specified (-  -real)");
    }
}

fn create_path_and_file(path: &str) -> Result<(),Box<Error>> {
//    use std::path::Path;

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }


    fs::write(path,b"")?;
    Ok(())
}

fn init_config(config: &Config) -> Result<(),Box<Error>>{
    create_path_and_file(&config.syllables_to_utf8_path)?;
    create_path_and_file(&config.word_database_path)?;
    create_path_and_file(&config.syllables_path)?;
    create_path_and_file(&config.occ_wanted_path)?;
    create_path_and_file(&config.syllable_valid_pos_path)?;

    Ok(())
}

#[cfg(test)]
mod tests;

fn soft_crash(msg:String){
    eprintln!("{}",msg);
    exit(-1);
}

fn main() -> Result<(), Box<Error>>{
    let mut app = prepare_app();
    let matches = app.clone().get_matches();

    let config = match Config::new(matches.value_of("config").unwrap()) {
        Ok(cfg) => cfg,
        _ => Config::default()
    };

    let mut my_lang = MyLang::new_with_config(config);

    if matches.is_present("init_config") {
        init_config(&my_lang.config)?;
        eprintln!("Config files initialized");
        return Ok(())
    }

    if !Path::new(matches.value_of("config").unwrap()).exists() {
        app.print_help()?;
        eprintln!();
        eprintln!("Config file not found, creating default");
        Config::default().create_file(matches.value_of("config").unwrap())?;
        eprintln!("Config file created");

    }

    if !validate_config(&my_lang) {
        eprintln!();
        eprintln!("Config is not valid, files are corrupted or missing");

        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("gen"){
        return handle_gen(matches,RandomEngine::new(my_lang,Box::new(CalculatedRandom::new())));
    }
    else if let Some(matches) = matches.subcommand_matches("db"){
        return handle_db(matches,&mut my_lang);
    }
    else if let Some(matches) = matches.subcommand_matches("config"){
        handle_config(matches,&mut my_lang);
    }
    else {
        app.print_help()?;
        println!();
    }

    Ok(())
}
