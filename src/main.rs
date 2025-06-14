use filesystem::collect_files;
use fuzzy_logic::fuzzy_matcher;
use nucleo::Matcher;
use ui::run_app;

mod filesystem;
mod fuzzy_logic;
mod search;
mod ui;
pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let files = collect_files(".", true)?;
    for file in &files {
        println!("{:?}", file);
    }

    let mut matcher = Matcher::default();

    let _ = run_app(&files, &mut matcher);

    Ok(())
}
