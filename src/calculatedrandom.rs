use crate::config::LangConfig;
use crate::rangen::RandomEngine;
use crate::{syllables, Result};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::HashMap;

pub struct CalculatedRandom {
    occ_count: HashMap<String, u32>,
    off_by_map: HashMap<String, f64>,
    sum: u32,
    rng: ThreadRng,
    // Select from top % results
    rng_range: f64,
}

impl CalculatedRandom {
    fn pull_syllable(&mut self, pos: usize, len: usize, cfg: &dyn LangConfig) -> String {
        self.off_by_map.clear();

        //How would offby change if we pulled this syllable
        for s_po in self.occ_count.keys() {
            // List through syllable that may be pulled
            let mut offby = 0.0; // How much values are off wanted values
            for (s_o, count) in &self.occ_count {
                // Compare syllable pulled s_po to others s_o
                let mut cadj = *count;
                if s_o == s_po {
                    // If this syllable was pulled
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

        for s in syllables::syllables_sorted_by_occurrence(&self.off_by_map) {
            // Iterate the sorted map
            if syllables::is_syllable_pos_valid(&s, pos, len, cfg) {
                //Until you find a valid char
                possible_results.push(s);
                possible_result_found = true;
            }

            if possible_results.len() >= possible_count {
                // We have enough results for our eng
                break;
            }
        }

        if !possible_result_found {
            // There are no valid results
            panic!("Can't construct any new random words");
        }

        // Possible results len used in case there were not enough results
        let result = possible_results[self.rng.gen_range(0, possible_results.len())].clone();

        if result.as_str() == "" {
            panic!("No suitable syllable could be pulled");
        }

        // Increase occurence of this syllable that we just pulled
        self.occ_count
            .entry(result.to_string())
            .and_modify(|e| *e += 1);

        // Increase total sum
        self.sum += 1;

        result
    }
}

impl RandomEngine for CalculatedRandom {
    fn create_words(
        &mut self,
        min_len: u32,
        max_len: u32,
        count: u32,
        cfg: &dyn LangConfig,
    ) -> Vec<String> {
        let mut result = Vec::with_capacity(count as usize);

        let mut rng = self.rng;
        for _nth_word in 0..count {
            let chosen_length = rng.gen_range(min_len, max_len + 1);
            let mut word = String::new();

            for pos in 0..chosen_length {
                word.push_str(&self.pull_syllable(pos as usize, chosen_length as usize, cfg));
            }

            result.push(word);
        }

        result
    }

    fn with_config(cfg: &dyn LangConfig) -> Result<Self> {
        let occ_count = syllables::db_syllable_occurrences_as_count(cfg)?;
        let sum = occ_count.values().sum(); // Must be done here because occ_count is given to CalculatedRandom after
        Ok(CalculatedRandom {
            occ_count,
            off_by_map: HashMap::new(),
            sum,
            rng: rand::thread_rng(),
            rng_range: 0.15,
        })
    }
}
