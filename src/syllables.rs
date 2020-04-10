use std::borrow::Borrow;
use crate::Result;
use crate::config::LangConfig;
use std::cmp;
use crate::error::LangErr::InvalidSyllable;

#[derive(PartialEq)]
pub enum SyllablePosition {
    Start,
    NotStart,
    End,
    NotEnd,
    Middle,
    NotMiddle,
    Any,
}

impl From<String> for SyllablePosition {
    fn from(string:String) -> Self {
        match string.to_lowercase().borrow() {
            "start" => SyllablePosition::Start,
            "notstart" => SyllablePosition::NotStart,
            "end" => SyllablePosition::End,
            "notend" => SyllablePosition::NotEnd,
            "middle" => SyllablePosition::Middle,
            "notmiddle" => SyllablePosition::NotMiddle,
            "any" => SyllablePosition::Any,
            _ => panic!("Unexpected syllable position")
        }
    }
}

pub fn split_into_syllables(word: &str, cfg: &dyn LangConfig) -> Result<Vec<String>> {
    assert!(word.trim() == word,"Whitespace in word is not allowed");

    let mut syllables = cfg.syllables().clone();
    syllables.sort_by(|a,b| a.len().cmp((&b.len()))); // Longer syllables must come first

    let max_len = syllables.iter()
        .map(String::len)
        .max()
        .unwrap(); // Longest syllable

    let min_len = syllables.iter()
        .map(String::len)
        .min()
        .unwrap(); // Shortest syllable

    let mut syl_res = Vec::new(); // Function result

    let mut char_count = 0; // Total chars in syl_res
    let mut char_len = max_len; // How many chars at once are we checking

    while char_count != word.len(){ // Go from last char checked to max syllable length or end of arr
        let part = &word[char_count..cmp::max(char_len,word.len()-1)];

        if cfg.syllables().iter().any(|s| s == part) { // Part is found in syllables (iter any used instead of contains because &String != &str)
            syl_res.push(part.to_string()); // Add it to result
            char_count += part.len(); // Increment char count
        }
        else if char_len-1 < min_len { // Can't decrease syl_size, it would be bellow min
            return Err(InvalidSyllable(part.to_string())) // Invalid syllable found
        }
        else {
            char_len -= 1; // Syllable with wanted  length not found, decreasing length
        }
    }

    Ok(syl_res)
}

pub fn is_syllable_pos_valid(syllable: &str, pos: usize, len: usize, cfg: &dyn LangConfig) -> bool {
    assert!(pos <= len,"Position of syllable can't be greater than word length");

    let last_index = len - 1; // Index of last char
    if let Some(position) = cfg.syllable_pos().get(syllable) {
        match(position,last_index) {
            (SyllablePosition::Start,0) => return true,
            (SyllablePosition::Start,_) => return false,
            (SyllablePosition::NotStart,0) => return false,
            (SyllablePosition::NotStart,_) => return true,
            (SyllablePosition::Middle,0) => return false,
            (SyllablePosition::Middle,pos) => return pos < last_index, // Is not last
            (SyllablePosition::NotMiddle,0) => return true, // Is first
            (SyllablePosition::NotMiddle,pos) => return pos == last_index, // Is last
            (SyllablePosition::End,pos) => return pos == last_index, // Is last
            (SyllablePosition::NotEnd,pos) => return pos != last_index, // Is not last
            (SyllablePosition::Any,_) => return true,
        }
    }
    else {
        return false;
    }
}

pub fn is_word_valid(word: &str, cfg: &dyn LangConfig) -> Result<()> {
    if word.trim() == "" { return Err(InvalidSyllable("".to_string()));} // Empty string

    let syllables = split_into_syllables(word,cfg)?;

    for (pos,syllable) in syllables.iter().enumerate() {
        if !is_syllable_pos_valid(syllable,pos,syllables.len(),cfg) {
            return Err(InvalidSyllable(syllable.to_string()));
        }
    }

    Ok(())
}

pub fn romanize(word: &str, cfg: &dyn LangConfig) -> Result<String> {
    let syllables = split_into_syllables(word,cfg)?;

    let mut result = String::with_capacity(word.len()); // Assume result will be at least same length

    for s in syllables {
        if let Some(r) = cfg.romanization().get(&s){
            result.push_str(r);
        }
        else {
            return Err(InvalidSyllable(s));
        }
    }

    Ok(result)
}

