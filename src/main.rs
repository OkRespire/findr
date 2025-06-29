use clap::Parser;
use filesystem::collect_files;
use nucleo::Matcher;
use ui::run_app;

mod filesystem;
mod highlight;
mod search;
mod ui;
pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Parser, Debug)]
#[command(name = "findr", version, about = "Fuzzy finder written in Rust")]
pub struct Args {
    #[arg(default_value = ".")]
    pub path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let files = collect_files(&args.path, true)?;
    let mut matcher = Matcher::default();
    let _ = run_app(&files, &mut matcher);

    Ok(())
}
