use std::{
    collections::HashMap,
    fmt::{self, Debug},
    fs::read_to_string,
};

use log::{debug, warn};

use crate::klo_options::KloOptions;
use rand::{seq::SliceRandom, thread_rng};

/// Pos is how positions in the [Blueprint] are assigned. The values are ordered as follows:
/// 1. Index of the [Row]
/// 2. Index of the [Key], going from left to right
/// 3. Index of the [Layer].
type Pos = (usize, usize, usize);

/// A plan for how to construct a [Layout]-instance.
pub type Blueprint = Vec<Row>;
type Row = Vec<Key>;
type Key = Vec<Layer>;
type Layer = String;

trait Blueprint_Helpers {
    fn debug_print(&self);
    fn get_key_pos(&mut self, needle: String) -> (usize, usize);
    fn replace_key(&mut self, new_key: String, old_key: String);
    fn set_key(&mut self, row: usize, key: usize, layer: usize, new_key: String);
}

impl Blueprint_Helpers for Blueprint {
    fn debug_print(&self) {
        for row in self {
            let mut keys = "".to_string();
            for key in row {
                keys = keys + &key.first().unwrap_or(&" ".to_string());
            }
            debug!("{}", keys);
        }
    }

    fn replace_key(&mut self, new_key: String, old_key: String) {
        let (row, key) = self.get_key_pos(old_key);
        self.set_key(row, key, 0, new_key);
    }

    fn set_key(&mut self, row: usize, key: usize, layer: usize, new_key: String) {
        self[row][key][layer] = new_key;
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

/// The Layout-struct, which will be used during optimization. It contains
/// 1. the layout's [Blueprint] and
/// 2. cashes of regularly used values.
///
/// Initialize a new layout-instance by calling
/// ```
/// Layout::from_blueprint(&blueprint);
/// ```
/// with a &[Blueprint] of your choosing.
pub struct Layout<'a> {
    /// The [Blueprint] of the layout.
    blueprint: Blueprint,

    /// A [HashMap] that caches for each character which finger is used to click on it.
    char_finger_dict: HashMap<String, &'a str>,

    /// A [HashMap] that caches for each character its corresponding position ([Pos]).
    char_pos_dict: HashMap<String, Pos>,

    /// A [HashMap] that caches for each position ([Pos]) whether that position ([Pos]) is typed using the left hand.
    pos_is_left_dict: HashMap<Pos, bool>,

    /// A [HashMap] that caches for each position ([Pos]) the corresponding character.
    pos_char_dict: HashMap<Pos, String>,
}

impl<'a> Layout<'a> {
    /// The "default constructor" for the [Layout]-struct.
    ///
    /// Input a (reference of a) [Blueprint] to let the function know what the layout should look like.
    pub fn from_blueprint<'b, 'c>(blueprint: &'b Blueprint) -> Layout<'c> {
        const RIGHT_HAND_LOWEST_INDEXES: [usize; 5] = [7, 6, 6, 7, 3];
    
        // The positions which are by default accessed by the given finger. 
        let FINGER_POS_LIST:[(&str, Vec<(usize, usize, usize)>); 10] = [
            ("Klein_L", vec![(0, 0, 0), (0, 1, 0), (0, 2, 0), (1, 0, 0), (1, 1, 0), (2, 0, 0), (2, 1, 0), (3, 0, 0), (3, 1, 0), (3, 2, 0), (4, 0, 0), (4, 1, 0)]), // Klein_L
            ("Ring_L", vec![(0, 3, 0), (1, 2, 0), (2, 2, 0), (3, 3, 0)]), // Ring_L
            ("Mittel_L", vec![(0, 4, 0), (1, 3, 0), (2, 3, 0), (3, 4, 0)]), // Mittel_L
            ("Zeige_L", vec![(0, 5, 0), (0, 6, 0), (1, 4, 0), (2, 4, 0), (3, 5, 0), (1, 5, 0), (2, 5, 0), (3, 6, 0)]), // Zeige_L
            ("Daumen_L", vec![(4, 2, 0), (4, 3, 0)]), // Daumen_L
            ("Daumen_R", vec![(4, 3, 0), (4, 4, 0)]), // Daumen_R
            ("Zeige_R", vec![(0, 7, 0), (0, 8, 0), (1, 6, 0), (2, 6, 0), (3, 7, 0), (1, 7, 0), (2, 7, 0), (3, 8, 0)]), // Zeige_R
            ("Mittel_R", vec![(0, 9, 0), (1, 8, 0), (2, 8, 0), (3, 9, 0)]), // Mittel_R
            ("Ring_R", vec![(0, 10, 0), (1, 9, 0), (2, 9, 0), (3, 10, 0)]), // Ring_R
            ("Klein_R", vec![(0, 11, 0), (0, 12, 0), (0, 13, 0), (1, 10, 0), (2, 10, 0), (3, 11, 0), (1, 11, 0), (2, 11, 0), (1, 12, 0), (2, 12, 0), (1, 13, 0), (2, 13, 0), (3, 12, 0), (4, 5, 0), (4, 6, 0), (4, 7, 0)]) // Klein_R
        ];
        let mut POS_TO_FINGER:HashMap<&(usize, usize, usize), &str> = HashMap::new();
        for (finger, positions) in &FINGER_POS_LIST{
            for pos in positions{
                POS_TO_FINGER.insert(pos, finger);
            }
        }
        
        let mut char_finger_dict: HashMap<String, &str> = HashMap::new();
        let mut char_pos_dict: HashMap<String, Pos> = HashMap::new();
        let mut pos_is_left_dict: HashMap<Pos, bool> = HashMap::new();
        let mut pos_char_dict: HashMap<Pos, String> = HashMap::new();

        for (row_idx, row) in blueprint.iter().enumerate() {
            
            // Only used to fill up self._pos_is_left_dict:
            let lowest_right_hand_idx = RIGHT_HAND_LOWEST_INDEXES[row_idx];
            
            
            for (key_idx , key) in row.iter().enumerate(){
                for (layer_idx, char) in key.iter().enumerate(){
                    
                    let pos: Pos = (row_idx, key_idx, layer_idx);
                    
                    // Check whether we even need to fill the HashMaps that take a character as a key.
                    let mut fill_char_dicts: bool = false;
                    if !char_finger_dict.contains_key(char){
                        fill_char_dicts = true;
                    } else if Self::is_position_cost_lower(char_pos_dict[char], pos) {
                        fill_char_dicts = true;
                    }
                    
                    if fill_char_dicts{
                        // Fill up _char_finger_dict
                        char_finger_dict.insert(char.to_owned(), match POS_TO_FINGER.get(&(pos.0, pos.1, 0)) {
                            Some(finger) => {finger},
                            None => {&""},
                        });
                        
                        // Fill up _char_pos_dict
                        char_pos_dict.insert(char.to_owned(), pos);
                    }                        
                    // Fill up _pos_is_left_dict
                    pos_is_left_dict.insert(pos,  lowest_right_hand_idx > key_idx);
                    
                    // Fill up _pos_char_dict
                    pos_char_dict.insert(pos,  char.to_owned());
                }
            }
        }
        Layout{
            blueprint: blueprint.to_owned(),
            char_finger_dict: char_finger_dict,
            char_pos_dict: char_pos_dict,
            pos_is_left_dict: pos_is_left_dict,
            pos_char_dict: pos_char_dict,
        }
    }

    /// Compares two positions ([Pos]) and returns whether the new position is better than the old one.
    fn is_position_cost_lower(old_pos: Pos, new_pos: Pos) -> bool{
        // use tripled layer cost, because it ignores the additional bigrams.
        const COST_LAYER_ADDITION: [u8; 6] = [0, 20, 9, 16, 29, 25];

        let old_cost = Self::single_key_position_cost(old_pos) + 2 * COST_LAYER_ADDITION[old_pos.2];
        let new_cost = Self::single_key_position_cost(new_pos) + COST_LAYER_ADDITION[new_pos.2];

        new_cost < old_cost
    }


    /// Get the cost of typing a single position ([Pos]).
    pub fn single_key_position_cost(pos: Pos) -> u8 {
        const COST_LAYER_ADDITION: [u8; 6] = [0, 20, 9, 16, 29, 25];
        let COST_PER_KEY: [Vec<u8>; 5] = [
            // The 0 values aren’t filled in at the moment.
            // Don’t put mutated keys there, otherwise the best keys will end up there!
            vec![80,    70,60,50,50,60,    60,50,50,50,50,60,70, 80], // Zahlenreihe (0)
            vec![24,    16,10, 5,12,17,    20,13, 5, 9,11,20,36,  0], // Reihe 1
            vec![9,      5, 3, 3, 3, 6,     6, 3, 3, 3, 5, 9,30, 6], // Reihe 2; enter low to make it preferred over the layer 4 enter.
            vec![20,16, 19,24,20,9,   30,  10, 8,22,22,17,       19],     // Reihe 3
            vec![0,0,0,                3           , 7, 0, 0, 0] // Reihe 4 mit Leertaste
        ];

        // TODO: Check whether pos ever can be null (or "Option -> None")
        // Arne has added at least two further checks but, so far, they seem unnecessary to me.
        
        /*if pos is None:  // not found
        return COST_PER_KEY_NOT_FOUND*/

        COST_PER_KEY[pos.0][pos.1] + COST_LAYER_ADDITION[pos.2]
    }
        
    pub fn from_args(options: &KloOptions) -> Layout {
        let mut layout: Layout = Self::get_base_layout(&options.base_layout);
        layout.debug_print();
        layout.merge_layout_string(options.starting_layout.as_ref());
        layout
    }

    fn get_base_layout(path: &Option<String>) -> Layout {
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

        let blueprint: Blueprint = serde_json::from_str(&json).unwrap();
        Layout::from_blueprint(&blueprint)
    }

    fn merge_layout_string(&mut self, layout_str: &str) -> Layout {
        let clean_layout_str = layout_str.replace(" ", "");
        let lines = clean_layout_str.split('\n');

        let mut blueprint = self.blueprint.clone();
        for (line_idx, line) in lines.enumerate() {
            let chars = line.chars();

            for (char_idx, char) in chars.enumerate() {
                blueprint.set_key(line_idx + 1, char_idx + 1, 0, char.into());
            }
        }
        Layout::from_blueprint(&blueprint)
    }

    pub fn debug_print(&self) {
        self.blueprint.debug_print();
    }

    pub fn get_randomized_variant(&self, alphabet: String, steps: u128) -> Layout {
        debug!("Creating a new randomized variant with {} steps.", steps);
        let mut blueprint = self.blueprint.clone();

        let mut old_alphabet = vec![];
        let mut new_alphabet = vec![];
        alphabet.chars().for_each(|c| {
            old_alphabet.push(c.to_string());
            new_alphabet.push(c.to_string());
        });
        new_alphabet.shuffle(&mut thread_rng());

        for (idx, new_char) in new_alphabet.iter().enumerate() {
            let old_char = &old_alphabet[idx];
            blueprint.replace_key(old_char.clone(), new_char.clone());
        }
        let new_layout = Layout::from_blueprint(&blueprint);

        debug!("{:?}", old_alphabet);
        debug!("{:?}", new_alphabet);
        self.debug_print();
        new_layout.debug_print();

        new_layout
    }
}
