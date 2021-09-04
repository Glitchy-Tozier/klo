use std::{
    collections::HashMap,
    fmt::{self, Debug},
    fs::read_to_string,
};

use log::{debug, warn};

use crate::klo_options::KloOptions;
use rand::{seq::SliceRandom, thread_rng};

type Layer = String;
type Key = Vec<Layer>;
type Row = Vec<Key>;
pub type Layout = Vec<Row>;

pub trait LayoutT {
    fn from_args(options: &KloOptions) -> Self;
    fn set_key(&mut self, row: usize, key: usize, layer: usize, new_key: String);
    fn get_base_layout(path: &Option<String>) -> Self;
    fn merge_layout_string(&mut self, layout: &str);
    fn debug_print(&self);
    fn get_randomized_variant(&self, alphabet: String, switches: u128) -> Self;
    fn set_new_key(&mut self, new_key: String, old_key: String);
    fn get_key_pos(&mut self, needle: String) -> (usize, usize);
}

impl LayoutT for Layout {
    fn from_args(options: &KloOptions) -> Self {
        let mut layout = Self::get_base_layout(&options.base_layout);
        layout.debug_print();
        layout.merge_layout_string(options.starting_layout.as_ref());
        layout
    }

    fn set_key(&mut self, row: usize, key: usize, layer: usize, new_key: String) {
        self[row][key][layer] = new_key;
    }

    fn get_base_layout(path: &Option<String>) -> Self {
        debug!("Reading base layout");
        let default_json = include_str!("../default_base_layout.json");
        let json = match path {
            Some(path) => {
                debug!("Reading json from argument with path {}.", path);
                read_to_string(path).unwrap()
            }
            None => {
                debug!("Assigning default layout (NEO)");
                default_json.to_string()
            }
        };

        serde_json::from_str(&json).unwrap()
    }

    fn merge_layout_string(&mut self, layout: &str) {
        let clean_lines = layout.replace(" ", "");
        let lines = clean_lines.split('\n');

        for (idx, line) in lines.enumerate() {
            let chars = line.chars();

            for (idy, char) in chars.enumerate() {
                self.set_key(idx + 1, idy + 1, 0, char.into());
            }
        }
    }

    fn debug_print(&self) {
        for row in self {
            let mut keys = "".to_string();
            for key in row {
                keys = keys + &key.first().unwrap_or(&" ".to_string());
            }
            debug!("{}", keys);
        }
    }

    fn get_randomized_variant(&self, alphabet: String, steps: u128) -> Self {
        debug!("Creating a new randomized variant with {} steps.", steps);
        let mut layout = self.clone();

        let mut old_alphabet = vec![];
        let mut new_alphabet = vec![];
        alphabet.chars().for_each(|c| {
            old_alphabet.push(c.to_string());
            new_alphabet.push(c.to_string());
        });
        new_alphabet.shuffle(&mut thread_rng());

        for (idx, new_char) in new_alphabet.iter().enumerate() {
            let old_char = &old_alphabet[idx];
            layout.set_new_key(old_char.clone(), new_char.clone());
        }

        debug!("{:?}", old_alphabet);
        debug!("{:?}", new_alphabet);
        self.debug_print();
        layout.debug_print();

        layout
    }

    fn set_new_key(&mut self, new_key: String, old_key: String) {
        let (row, key) = self.get_key_pos(old_key);
        self.set_key(row, key, 0, new_key);
    }

    fn get_key_pos(&mut self, needle: String) -> (usize, usize) {
        let mut row_index = 0;
        let mut key_index = 0;

        for (inr, row) in self.iter().enumerate() {
            for (ink, key) in row.iter().enumerate() {
                let default_key = "not_set".to_string();
                let key = key.first().unwrap_or(&default_key);
                if key == &needle {
                    row_index = inr;
                    key_index = ink;
                }
            }
        }

        debug!("Found key {} in {} x {}", needle, row_index, key_index);

        (row_index, key_index)
    }
}
