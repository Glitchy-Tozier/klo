use lazy_static::lazy_static;
use log::{debug, warn};
use rayon::iter::ParallelIterator;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{fs, vec};

/**
NGrams contains ngrams from various sources in raw and weighted
form and can export them to the simple (1gramme.txt, 2gramme.txt,
3gramme.txt) form with a given number of total keystrokes.
*/
pub struct NGrams {
    pub letters: Vec<(String, f64)>,
    pub bigrams: Vec<(String, f64)>,
    pub trigrams: Vec<(String, f64)>,
}

struct RawNGrams {
    weight: f64,
    letters: Vec<(String, f64)>,
    bigrams: Vec<(String, f64)>,
    trigrams: Vec<(String, f64)>,
}

struct NormalizedNGrams {
    weight: f64,
    letters: Vec<(String, f64)>,
    bigrams: Vec<(String, f64)>,
    trigrams: Vec<(String, f64)>,
}

impl NGrams {
    pub fn from_config(path: &str) -> Result<NGrams, String> {
        debug!("Trying to open ngrams config file {}", path);
        let config = fs::read_to_string(path).expect("Unable to open given ngrams config file.");

        /*
        Originally python could parse v0.0 ngrams, but they are not part of the repo anymore
        so we only parse v0.1 in this rewrite.
        */

        // Remove comment lines
        let lines: Vec<&str> = config
            .lines()
            .filter(|line| !line.starts_with("#"))
            .collect();

        let raw_vec: Vec<_> = lines
            .into_par_iter()
            .map(|line| Self::work_ngrams_config_line(line))
            .collect();

        let normalized: Vec<_> = raw_vec
            .into_iter()
            .filter(|raw| raw.is_some())
            .map(|raw| {
                let raw = raw.as_ref().unwrap();
                Self::normalize_ngrams(raw)
            })
            .collect();

        Ok(Self::collect_normalized_ngrams(normalized))
    }

    fn collect_normalized_ngrams(normalized: Vec<NormalizedNGrams>) -> Self {
        let mut letter_weight = HashMap::new();
        let mut bigram_weight = HashMap::new();
        let mut trigram_weight = HashMap::new();

        for ngram in normalized {
            for (letter, num) in ngram.letters {
                *letter_weight.entry(letter).or_insert(0.0) += num * ngram.weight;
            }

            for (bigram, num) in ngram.bigrams {
                *bigram_weight.entry(bigram).or_insert(0.0) += num * ngram.weight;
            }

            for (trigram, num) in ngram.trigrams {
                *trigram_weight.entry(trigram).or_insert(0.0) += num * ngram.weight;
            }
        }

        let mut letters = vec![];
        letter_weight.into_iter().for_each(|(letter, num)| {
            letters.push((letter, num));
        });

        let mut bigrams = vec![];
        bigram_weight.into_iter().for_each(|(bigram, num)| {
            bigrams.push((bigram, num));
        });

        let mut trigrams = vec![];
        trigram_weight.into_iter().for_each(|(trigram, num)| {
            trigrams.push((trigram, num));
        });

        NGrams {
            letters,
            bigrams,
            trigrams,
        }
    }

    fn normalize_ngrams(ngrams: &RawNGrams) -> NormalizedNGrams {
        let sum_letters: f64 = ngrams.letters.iter().fold(0.0, Self::fold_ngrams);
        let sum_bigrams: f64 = ngrams.bigrams.iter().fold(0.0, Self::fold_ngrams);
        let sum_trigrams: f64 = ngrams.trigrams.iter().fold(0.0, Self::fold_ngrams);
        let total = sum_letters + sum_bigrams + sum_trigrams;

        let normalized_letters: Vec<_> = ngrams
            .letters
            .iter()
            .map(|(letter, number)| (letter.clone(), *number as f64 / total as f64))
            .collect();

        let normalized_bigrams: Vec<_> = ngrams
            .bigrams
            .iter()
            .map(|(letter, number)| (letter.clone(), *number as f64 / total as f64))
            .collect();

        let normalized_trigrams: Vec<_> = ngrams
            .trigrams
            .iter()
            .map(|(letter, number)| (letter.clone(), *number as f64 / total as f64))
            .collect();

        NormalizedNGrams {
            weight: ngrams.weight,
            letters: normalized_letters,
            bigrams: normalized_bigrams,
            trigrams: normalized_trigrams,
        }
    }

    fn fold_ngrams(sum: f64, ngram: &(String, f64)) -> f64 {
        sum + ngram.1
    }

    fn work_ngrams_config_line(line: &str) -> Option<RawNGrams> {
        let line_array = line.split(" ");
        let parts: Vec<&str> = line_array
            .filter(|part| part.to_string() != "".to_string())
            .collect();

        let weight = parts[0].parse::<f64>().unwrap();
        let datatype = parts[1];
        let datapath = parts[2];

        debug!(
            "Read config line => weight: {} ---- type: {} ---- path: {} ",
            weight, datatype, datapath
        );

        if datatype == "text" {
            Some(Self::parse_text_ngrams(weight, datapath))
        } else if datatype == "pregenerated" {
            let paths: Vec<&str> = datapath.split(";").collect();
            Some(Self::parse_pregenerated_ngrams(
                weight, paths[0], paths[1], paths[2],
            ))
        } else {
            warn!("Unsupported data type {}", datatype);
            //todo!("Implement error for unsupported data types.");
            None
        }
    }

    fn parse_pregenerated_ngrams(
        weight: f64,
        letters_path: &str,
        bigrams_path: &str,
        trigrams_path: &str,
    ) -> RawNGrams {
        let letters = Self::read_pregenerated_file(letters_path.to_string());
        let bigrams = Self::read_pregenerated_file(bigrams_path.to_string());
        let trigrams = Self::read_pregenerated_file(trigrams_path.to_string());
        RawNGrams {
            weight,
            letters,
            bigrams,
            trigrams,
        }
    }

    fn read_pregenerated_file(path: String) -> Vec<(String, f64)> {
        let contents = fs::read_to_string(path).unwrap();

        let mut data = vec![];

        for line in contents.lines() {
            let cleaned_line = line.replace("\u{feff}", "");
            let line_array = cleaned_line.split(" ");
            let parts: Vec<&str> = line_array
                .filter(|part| part.to_string() != "".to_string())
                .collect();

            let mut letters = parts.last().unwrap().to_string();

            if line.chars().last().unwrap() == ' ' {
                if letters == parts.first().unwrap().to_string() {
                    letters = line.chars().last().unwrap().to_string();
                } else {
                    letters += &line.chars().last().unwrap().to_string();
                }
            }

            if parts.len() == 2 || (parts.len() == 1 && line.chars().last().unwrap() == ' ') {
                let weight = parts.first().unwrap();
                let number = weight.parse::<f64>().unwrap();
                data.push((letters, number))
            }
        }
        data
    }

    fn parse_text_ngrams(weight: f64, path: &str) -> RawNGrams {
        let f = File::open(path).unwrap();
        let mut reader = BufReader::new(f);
        let mut buf = vec![];

        let mut letters: HashMap<String, f64> = HashMap::new();
        let mut bigrams: HashMap<String, f64> = HashMap::new();
        let mut trigrams: HashMap<String, f64> = HashMap::new();

        let mut bigram_char = None;
        let mut trigram_char = None;

        while let Ok(_) = reader.read_until(b'\n', &mut buf) {
            if buf.is_empty() {
                break;
            }
            let line = String::from_utf8_lossy(&buf);

            lazy_static! {
                static ref UPPER_CASE_REGEX: Regex = Regex::new(r"[A-Z]").unwrap();
            }

            let line = UPPER_CASE_REGEX.replace_all(&line, "⇧");

            let chars = line.chars();

            for letter in chars {
                if letter != '⇧' {
                    *letters.entry(letter.to_string()).or_insert(0.0) += 1.0;
                }

                if let Some(bigram_char) = bigram_char {
                    *bigrams
                        .entry(format!("{}{}", letter, bigram_char))
                        .or_insert(0.0) += 1.0;

                    if let Some(trigram_char) = trigram_char {
                        *trigrams
                            .entry(format!("{}{}{}", letter, bigram_char, trigram_char))
                            .or_insert(0.0) += 1.0;
                    }

                    trigram_char = Some(bigram_char);
                }

                bigram_char = Some(letter.to_string());
            }

            buf.clear();
        }

        let mut trigrams_drained: Vec<_> = trigrams.iter().collect();
        let trigrams_final_length = trigrams_drained.len().saturating_sub(1);
        trigrams_drained.truncate(trigrams_final_length);
        trigrams_drained.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        trigrams_drained.reverse();

        let mut letters_vec = vec![];

        for (letter, count) in letters.iter() {
            letters_vec.push((letter.clone(), count.clone()))
        }
        letters_vec.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        letters_vec.reverse();

        let mut bigrams_vec = vec![];
        let bigrams_final_length = bigrams.len().saturating_sub(2);
        bigrams_vec.sort_by(|a: &(String, f64), b: &(String, f64)| a.1.partial_cmp(&b.1).unwrap());
        bigrams_vec.reverse();

        for (bigram, count) in bigrams.iter() {
            bigrams_vec.push((bigram.clone(), count.clone()))
        }

        // Remove the last two bigrams because they just get filled up with the same char.
        bigrams_vec.truncate(bigrams_final_length);

        let mut trigrams_vec = vec![];
        let trigrams_final_length = trigrams_drained.len().saturating_sub(1);

        for (trigram, count) in trigrams.iter() {
            trigrams_vec.push((trigram.clone(), count.clone()));
        }

        // Remove the last trigram because it just gets filled up with the same char.
        trigrams_vec.truncate(trigrams_final_length);

        RawNGrams {
            weight,
            letters: letters_vec,
            bigrams: bigrams_vec,
            trigrams: trigrams_vec,
        }
    }
}
