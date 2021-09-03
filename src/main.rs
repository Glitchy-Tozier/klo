use clap::Clap;
use klo_options::KloOptions;
use log::{debug, LevelFilter};
use simple_logger::SimpleLogger;

mod check_neo;
mod klo_options;
mod layout;
mod ngrams;

fn main() {
    let mut options = KloOptions::parse();
    options.post_parse_checks();

    if options.quiet {
        SimpleLogger::new()
            .with_level(LevelFilter::Warn)
            .init()
            .unwrap();
    } else if options.verbose {
        SimpleLogger::new()
            .with_level(LevelFilter::Trace)
            .init()
            .unwrap();
        debug!("Verbose mode is on - going to talk to you a lot.")
    } else {
        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .init()
            .unwrap();
    }

    check_neo::evolve_a_layout(&options);
}
