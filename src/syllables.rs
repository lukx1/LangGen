use crate::config::LangConfig;
use crate::error::LangErr::InvalidSyllable;
use crate::Result;
use std::borrow::Borrow;
use std::cmp;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Represents where in a word a syllable is
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
    fn from(string: String) -> Self {
        match string.to_lowercase().borrow() {
            "start" => SyllablePosition::Start,
            "notstart" => SyllablePosition::NotStart,
            "end" => SyllablePosition::End,
            "notend" => SyllablePosition::NotEnd,
            "middle" => SyllablePosition::Middle,
            "notmiddle" => SyllablePosition::NotMiddle,
            "any" => SyllablePosition::Any,
            _ => panic!("Unexpected syllable position"),
        }
    }
}

/// Splits a single word into syllables. Returns
/// error if word can't be split
pub fn split_into_syllables(word: &str, cfg: &dyn LangConfig) -> Result<Vec<String>> {
    assert!(word.trim() == word, "Whitespace in word is not allowed");

    let mut syllables = cfg.syllables().clone();
    syllables.sort_by(|a, b| a.len().cmp(&b.len())); // Longer syllables must come first

    let max_len = syllables
        .iter() // Unicode characters may be longer than thay appear,
        .map(String::len) // this is working as intended
        .max()
        .unwrap(); // Longest syllable

    let min_len = syllables.iter().map(String::len).min().unwrap(); // Shortest syllable

    let mut syl_res = Vec::new(); // Function result

    let mut char_count = 0; // Total chars in syl_res
    let mut char_len = max_len; // How many chars at once are we checking

    while char_count != word.len() {
        // Go from last char checked to max syllable length or end of arr
        let part = &word[char_count..(char_count + cmp::min(char_len, word.len()))];

        if cfg.syllables().iter().any(|s| s == part) {
            // Part is found in syllables (iter any used instead of contains because &String != &str)
            syl_res.push(part.to_string()); // Add it to result
            char_count += part.len(); // Increment char count
        } else if char_len - 1 < min_len {
            // Can't decrease syl_size, it would be bellow min
            return Err(InvalidSyllable(part.to_string())); // Invalid syllable found
        } else {
            char_len -= 1; // Syllable with wanted  length not found, decreasing length
        }
    }

    Ok(syl_res)
}

/// Validates if a chosen syllable is valid at a specified position with a select length
pub fn is_syllable_pos_valid(syllable: &str, pos: usize, len: usize, cfg: &dyn LangConfig) -> bool {
    assert!(
        pos <= len,
        "Position of syllable can't be greater than word length"
    );

    let last_index = len - 1; // Index of last char
    if let Some(position) = cfg.syllable_pos().get(syllable) {
        match (position, last_index) {
            (SyllablePosition::Start, 0) => true,
            (SyllablePosition::Start, _) => false,
            (SyllablePosition::NotStart, 0) => false,
            (SyllablePosition::NotStart, _) => true,
            (SyllablePosition::Middle, 0) => false,
            (SyllablePosition::Middle, pos) => pos < last_index, // Is not last
            (SyllablePosition::NotMiddle, 0) => true,            // Is first
            (SyllablePosition::NotMiddle, pos) => pos == last_index, // Is last
            (SyllablePosition::End, pos) => pos == last_index,   // Is last
            (SyllablePosition::NotEnd, pos) => pos != last_index, // Is not last
            (SyllablePosition::Any, _) => true,
        }
    } else {
        false
    }
}

//TODO:lukx this function is useful but the user can't call it
///// Validates every syllable in a single words if it is valid. Empty string
///// or whitespace will return an error
//pub fn is_word_valid(word: &str, cfg: &dyn LangConfig) -> Result<()> {
//    if word.trim() == "" { return Err(InvalidSyllable("".to_string()));} // Empty string
//
//    let syllables = split_into_syllables(word,cfg)?;
//
//    for (pos,syllable) in syllables.iter().enumerate() {
//        if !is_syllable_pos_valid(syllable,pos,syllables.len(),cfg) {
//            return Err(InvalidSyllable(syllable.to_string()));
//        }
//    }
//
//    Ok(())
//}
/// Replaces characters in word with their romanized equivalent.
/// Returns error if word can't be split or a syllable in word can't be
/// found in database
pub fn romanize(word: &str, cfg: &dyn LangConfig) -> Result<String> {
    let syllables = split_into_syllables(word, cfg)?;

    let mut result = String::with_capacity(word.len()); // Assume result will be at least same length

    for s in syllables {
        if let Some(r) = cfg.romanization().get(&s) {
            result.push_str(r);
        } else {
            return Err(InvalidSyllable(s));
        }
    }

    Ok(result)
}

/// Sorts a HashMap of syllables by their occurrance and returns
/// a descending vector of them
pub fn syllables_sorted_by_occurrence(syllab: &HashMap<String, f64>) -> Vec<String> {
    let mut syllables: Vec<String> = syllab.keys().cloned().collect(); // Collect only keys (syllables)

    syllables.sort_by(|a,b|  // Sort them by their position inside perc, desc
        comp_f64(*syllab.get(b).unwrap(),*syllab.get(a).unwrap()));
    // Unwrap is safe because we just took them from perc HashMap

    syllables
}

/// Sorts a HashMap of syllables by their occurrence and returns a descending vector of
/// syllables by percentage of occurrance
pub fn syllables_by_occurrence_desc(syllables: &HashMap<String, f64>) -> Vec<(String, f64)> {
    let ss = syllables_sorted_by_occurrence(syllables);
    let mut result = Vec::with_capacity(ss.len());

    for syllable in ss {
        let oc = *syllables.get(&syllable).unwrap();
        result.push((syllable, oc));
    }

    result
}

/// Counts unique syllables in database and returns their count. Syllables that are not present in
/// the database but are present in syllables list will be added with count of 0.
/// Returns error if any word in database can not be split into syllables.
pub fn db_syllable_occurrences_as_count(cfg: &dyn LangConfig) -> Result<HashMap<String, u32>> {
    use crate::syllables::*;

    let mut count: HashMap<String, u32> = HashMap::new();

    for word in cfg.database() {
        let syllables = split_into_syllables(word, cfg)?;
        for syllable in syllables {
            count.entry(syllable).and_modify(|e| *e += 1).or_insert(1); // Increment by 1 or if not found, set to 1
        }
    }

    if count.len() < cfg.syllables().len() {
        // Not every syllable is in results
        for syllable in cfg.syllables() {
            count
                .entry(syllable.to_string()) // Prob should not clone
                .or_insert(0); // If not found insert 0 percent
        }
    }

    Ok(count)
}

/// Converts count of syllables in into percentages. Will not work correctly if syllables with no
/// occurrence but presence in DB are not in @count.
pub fn db_syllable_occurrences_as_percentage(count: &HashMap<String, u32>) -> HashMap<String, f64> {
    let total = count.iter().map(|(_, v)| *v).sum::<u32>() as f64; // Calculate total count

    let mut result = HashMap::new();

    count.iter().for_each(|(syllable, count)| {
        // For every syllable calculate percentage of total
        result.insert(syllable.to_string(), *count as f64 / total);
    });

    result
}

fn comp_f64(a: f64, b: f64) -> Ordering {
    if a < b {
        return Ordering::Less;
    } else if a > b {
        return Ordering::Greater;
    }
    Ordering::Equal
}
