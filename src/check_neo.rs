use crate::layout::Layout;
use log::debug;
use std::convert::TryInto;

use crate::{klo_options::KloOptions, ngrams::NGrams};
/// Evolve a layout by selecting the fittest of random mutations step by step.
pub fn evolve_a_layout(options: &KloOptions) {
    let mut ngram_data = NGrams::from_config(&options.ngrams_config).unwrap();

    if options.limit_ngrams > 0 {
        ngram_data
            .letters
            .truncate(options.limit_ngrams.try_into().unwrap());
        ngram_data
            .bigrams
            .truncate(options.limit_ngrams.try_into().unwrap());
        ngram_data
            .trigrams
            .truncate(options.limit_ngrams.try_into().unwrap());
    }

    if options.prerandomize > 0 {
        debug!("Doing {} prerandomization switches.", options.prerandomize);
        let layout = Layout::from_args(&options);
        layout.debug_print();
        let new = layout.get_randomized_variant(options.alphabet.clone(), options.steps);
    }
}
