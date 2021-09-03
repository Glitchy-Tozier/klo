use std::{fmt, fs::read_to_string};

use log::debug;

use crate::klo_options::KloOptions;

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
                self.set_key(idx + 1, idy +1, 0, char.into());
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
}
