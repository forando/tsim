mod csv_extractor;
mod console_writer;
mod similarity_analyzer;
mod threadpool;
mod utils;

use structopt::StructOpt;
use anyhow::Result;

fn main() -> Result<()> {
    let opt  = process_opts();

    let records = csv_extractor::parse_csv(&opt.path)?;

    let similarities = similarity_analyzer::run(records);

    if opt.display {
        println!("{}", similarities);
    } else {
        println!("{}", similarities.to_string())
    }

    Ok(())
}

/// Calculates similarities between texts with Jaccard Index(https://en.wikipedia.org/wiki/Jaccard_index)
#[derive(Debug, StructOpt)]
struct Opt {
    /// The path to the file to read
    #[structopt(short = "p", long = "path", parse(from_os_str))]
    pub path: std::path::PathBuf,

    /// Displays similarities distribution
    #[structopt(long)]
    pub display: bool
}

fn process_opts() -> Opt {
    Opt::from_args()
}
