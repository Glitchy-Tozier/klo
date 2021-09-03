use std::cmp::max;

use clap::Clap;

// Keyboard Layout Optimizer based on https://hg.sr.ht/~arnebab/evolve-keyboard-layout/browse?rev=tip
#[derive(Clap, Debug)]
#[clap(name = "klo")]
pub struct KloOptions {
    /// The number of new layouts to create. Can be overwritten with the -n parameter. 500 should have a 50% chance of finding the best possible layout (the global minimum).
    #[clap(short = 'n', long, default_value = "500")]
    pub num_layouts: u128,

    /// The output filename. Can be overwritten with the -o parameter.
    #[clap(short = 'o', long, default_value = "output.txt")]
    pub filename: String,

    /// The number of random evolution steps to take.
    #[clap(long, default_value = "10000")]
    pub steps: u128,

    /// The number of random mutations to do before the evolution to get a random layout.
    #[clap(long, default_value = "3000")]
    pub prerandomize: u128,

    /// Should we always do the locally best step? (very slow and *not* optimal)
    #[clap(long, parse(try_from_str), default_value = "false")]
    pub controlled: bool,

    /// Should we avoid giving information on the shell? (Windows users enable this, cause the default shell can’t take Unicode)
    #[clap(long)]
    pub quiet: bool,

    /// Should we give additional statistics for the final layout?
    #[clap(long)]
    pub verbose: bool,

    /// Should we finalize the layout with as many controlled steps as needed, so a single keyswitch can’t improve it further?
    #[clap(long, parse(try_from_str), default_value = "true")]
    pub controlled_tail: bool,

    /// Should we use annealing? How many steps? Per step it adds one switch, so anneal 5 starts with 6 switches aka changing half the layout (12 keys).
    #[clap(long, default_value = "5")]
    pub anneal: u128,

    /// The number of iterations to spend in one anneal level. The first anneal * anneal_step iterations are spent in simulated annealing.
    #[clap(long, default_value = "1000")]
    pub anneal_step: u128,

    /// Should we limit the number of ngrams? A value of 3000 should still be safe to quickly see results without getting unreasonable layouts. Use 0 for no-limit.
    #[clap(long, default_value = "0")]
    pub limit_ngrams: u128,

    /// The layout to use as base for mutations. If you want a given starting layout, also set prerandomize = 0.
    #[clap(long, default_value = "bmuaz kdflvjß\ncriey ptsnh⇘\nxäüoö wg,.q")]
    pub starting_layout: String,

    /// Path to your ngrams.config
    #[clap(long, default_value = "ngrams.config")]
    pub ngrams_config: String,

    /// The alphabet to use
    #[clap(long, default_value = "abcdefghijklmnopqrstuvwxyzäöüß")]
    pub alphabet: String,

    /// Path to your base_layout.json. If non is supplied the neo layout is used.
    #[clap(long)]
    pub base_layout: Option<String>,
}

impl KloOptions {
    pub fn post_parse_checks(&mut self) {
        // ensure that at most half the time is spent annealing
        if self.anneal * self.anneal_step > self.steps {
            let half_steps = 0.5 * self.steps as f64;
            let calculated_anneals = half_steps / (1 + self.anneal) as f64;
            self.anneal_step = max(1, calculated_anneals as u128);
        }
    }
}
