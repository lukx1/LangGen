use crate::config::LangConfig;
use crate::rangen::RandomEngine;
use crate::{syllables, Result};
use rand::prelude::ThreadRng;
use rand::Rng;

/// Returns words and syllables randomly.
/// Will panic if no words can be generated
pub struct RealRandom {
    rng: ThreadRng,
    max_tries: u32,
}

impl RealRandom {
    fn pull_syllable(&mut self, pos: usize, len: usize, cfg: &dyn LangConfig) -> String {
        let mut tries = 0;
        while tries < self.max_tries {
            let syllable = cfg
                .syllables() // Pull random syllable
                .get(self.rng.gen_range(0, cfg.syllables().len()))
                .unwrap(); // Can never be out of bounds

            if syllables::is_syllable_pos_valid(&syllable, pos, len, cfg) {
                // Check if it is valid
                return syllable.to_string(); // Return if OK
            } // Otherwise try again

            tries += 1; // Keep at bottom
        }

        panic!(
            "Could not get a suitable syllable in {} tries",
            self.max_tries
        )
    }
}

impl RandomEngine for RealRandom {
    fn create_words(
        &mut self,
        min_len: u32,
        max_len: u32,
        count: u32,
        cfg: &dyn LangConfig,
    ) -> Vec<String> {
        let mut result = Vec::with_capacity(count as usize);

        for _nth_word in 0..count {
            let chosen_length = self.rng.gen_range(min_len, max_len + 1);
            let mut word = String::new();

            for pos in 0..chosen_length {
                word.push_str(&self.pull_syllable(pos as usize, chosen_length as usize, cfg));
            }

            result.push(word);
        }

        result
    }

    fn with_config(_cfg: &dyn LangConfig) -> Result<RealRandom>
    where
        Self: Sized,
    {
        Ok(RealRandom {
            rng: rand::thread_rng(),
            max_tries: 100,
        })
    }
}
