use crate::{Result, syllables};
use std::collections::HashMap;
use crate::rangen::RandomEngine;
use crate::config::LangConfig;
use std::cmp::Ordering;
use rand::{thread_rng, Rng};
use rand::rngs::ThreadRng;

struct CalculatedRandom {
    occ_count: HashMap<String,u32>,
    off_by_map: HashMap<String,f64>,
    sum: u32,
    rng: ThreadRng,
    // Select from top % results
    rng_range: f64
}

impl CalculatedRandom {
    fn db_syllable_occurrences_as_count(cfg: &dyn LangConfig) -> Result<HashMap<String,u32>>{
        use crate::syllables::*;

        let mut count: HashMap<String,u32> = HashMap::new();

        for word in cfg.database() {
            let syllables = split_into_syllables(word,cfg)?;
            for syllable in syllables {
                count.entry(syllable)
                    .and_modify(|e| *e += 1)
                    .or_insert(1); // Increment by 1 or if not found, set to 1
            }
        }

        if count.len() < cfg.syllables().len() { // Not every syllable is in results
            for syllable in cfg.syllables() {
                count.entry(syllable.to_string()) // Prob should not clone
                    .or_insert(0); // If not found insert 0 percent
            }
        }

        Ok(count)
    }
    fn db_syllable_occurrences_as_percentage(count: &HashMap<String,u32>, cfg: &dyn LangConfig) -> Result<HashMap<String,f64>> {
        assert!(count.len() == cfg.syllables().len(),"Not enough syllables were counted"); // All syllables, even those with no occourances must be in count map

        let total = count.iter().map(|(k,v)| *v).sum::<u32>() as f64; // Calculate total count

        let mut result = HashMap::new();

        count.iter()
            .for_each(|(syllable,count)| { // For every syllable calculate percentage of total
                result.insert(syllable.to_string(),*count as f64 / total);});

        Ok(result)
    }
    fn comp_f64(a: &f64, b: &f64) -> Ordering {
        if a < b {
            return Ordering::Less;
        } else if a > b {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
    fn db_syllable_sorted_by_occupancy_desc(perc: &HashMap<String,f64>) -> Vec<String> {
        let mut syllables:Vec<String> = perc.keys().cloned().collect(); // Collect only keys (syllables)

        syllables.sort_by(|a,b|  // Sort them by their position inside perc, desc
            CalculatedRandom::comp_f64(perc.get(a).unwrap(),perc.get(b).unwrap()));
        // Unwrap is safe because we just took them from perc HashMap

        syllables
    }
    fn pull_syllable(&mut self, pos:usize, len:usize, cfg: &dyn LangConfig) -> String{
        self.off_by_map.clear();

        //How would offby change if we pulled this syllable
        for s_po in self.occ_count.keys() { // List through syllable that may be pulled
            let mut offby = 0.0; // How much values are off wanted values
            for (s_o,count) in &self.occ_count { // Compare syllable pulled s_po to others s_o
                let mut cadj= *count;
                if s_o == s_po { // If this syllable was pulled
                    cadj += 1; // Add one
                }
                //Otherwise it stays the same

                let adj_real = cadj as f64 / self.sum as f64; // Calculate frequency

                offby += (*cfg.wanted().get(s_o).unwrap() - (adj_real)).abs(); // Calculate offby for this syllable
            }
            self.off_by_map.insert(s_po.clone(), offby); // Add this option to sorted map
        }

        // How many possibilities should we prepare for considering our rng range
        let possible_count = ((self.occ_count.len() as f64) * self.rng_range) as usize;

        // Not every syllable can be used at this position
        let mut possible_results = Vec::with_capacity(possible_count);
        let mut possible_result_found = false;

        for s in CalculatedRandom::db_syllable_sorted_by_occupancy_desc(&self.off_by_map){ // Iterate the sorted map
            if syllables::is_syllable_pos_valid(&s,pos,len,cfg) {//Until you find a valid char
                possible_results.push(s);
                possible_result_found = true;
            }

            if possible_results.len() >= possible_count { // We have enough results for our eng
                break;
            }
        }

        if !possible_result_found { // There are no valid results
            panic!("Can't construct any new random words");
        }

        // Possible results len used in case there were not enough results
        let result = possible_results[self.rng.gen_range(0,possible_results.len())].clone();

        if result.as_str() == "" {
            panic!("No suitable syllable could be pulled");
        }

        // Increase occurence of this syllable that we just pulled
        self.occ_count.entry(result.to_string()).and_modify(|e| *e += 1);

        // Increase total sum
        self.sum += 1;

        result
    }
}


impl RandomEngine for CalculatedRandom {
    fn create_words(&mut self, min_len: u32, max_len: u32, count: u32,cfg: &dyn LangConfig) -> Vec<String> {
        let mut result = Vec::with_capacity(count as usize);

        let mut rng = rand::thread_rng();
        for nth_word in 0..count {

            let chosen_length = rng.gen_range(min_len,max_len+1);
            let mut word = String::new();
            let mut pos = 0;

            for nth_syllable in 0..chosen_length {
                word.push_str(&self.pull_syllable(pos,chosen_length as usize,cfg));
                pos += 1;
            }

            result.push(word);
        }

        result
    }

    fn with_config(cfg: &dyn LangConfig) -> Result<Self> {
        let occ_count = CalculatedRandom::db_syllable_occurrences_as_count(cfg)?;
        let sum = occ_count.values().sum(); // Must be done here because occ_count is given to CalculatedRandom after
        Ok(
            CalculatedRandom {
                occ_count,
                off_by_map: HashMap::new(),
                sum,
                rng: rand::thread_rng(),
                rng_range: 0.15
            }
        )

    }
}