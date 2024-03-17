use std::{env, io::{self, Write}};
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    io::stdout().write_all({
        env::args()
            .collect::<Vec<String>>()[1..]
            .join(" ") + "\n"
    }.as_bytes())?;
    Ok(())
}
