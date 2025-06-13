use filesystem::collect_files;
use fuzzy_logic::fuzzy_matcher;
use ui::run_app;

mod filesystem;
mod fuzzy_logic;
mod search;
mod ui;
pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let files = collect_files(".", true)?;
    for file in files {
        println!("{:?}", file);
    }

    let res = fuzzy_matcher("abc", "a_big_camel");
    println!("{:?}", res);

    run_app();

    Ok(())
}
