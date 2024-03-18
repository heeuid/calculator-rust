use clap::Parser;
use std::{io::{self, Write}, fs, path::Path, os::unix::fs::FileTypeExt};
use anyhow::anyhow;

#[derive(Parser, Debug)]
#[command(version, about = None, long_about =
    "List information about the FILEs (the current directory by default)."
)]
struct Args {
    // /// Explanation
    // #[arg(short = 'v', long, default_value_t = false, verbatim_doc_comment)]
    // show_nonprinting: bool,

    /// Paths: directory or file
    #[clap(value_parser, required = false, verbatim_doc_comment)]
    paths: Vec<String>, }

fn deal_with_file(output: &mut Vec<String>, path: &Path, meta: &fs::Metadata, dir: bool) -> anyhow::Result<()> {
    let file_type = meta.file_type();
    let file_name = match path.file_name() {
        Some(filename) => match filename.to_str() {
            Some(file_name) => file_name,
            None => return Err(anyhow!("Failed to change OsStr to str")),
        }
        None => return Err(anyhow!("Failed to extract filename")),
    };
    if file_type.is_file() { // 0. regular file
        output.push(file_name.to_string());
    } else if dir && file_type.is_dir() { // 1. directory
        output.push(format!("{}/", file_name));
    } else if file_type.is_symlink() { // 2. symbolic link
        output.push(format!("{}->", file_name));
    } else if file_type.is_socket() { // 3. socket
        output.push(format!("s->{}", file_name));
    } else if file_type.is_fifo() { // 4. pipe
        output.push(format!("p->{}", file_name));
    } else if file_type.is_block_device() { // 5. block dev
        output.push(format!("b->{}", file_name));
    } else if file_type.is_char_device() { // 6. char dev
        output.push(format!("c->{}", file_name));
    } else {
        return Err(anyhow!("Unknown file type"));
    }
    Ok(())
}

fn enter_dir(output: &mut Vec<String>, path: &Path) -> anyhow::Result<()> {
    for entry in fs::read_dir(path)? {
        let path = match entry {
            Ok(component) => component,
            Err(e) => {
                println!("ls {:?}: {}", path, e);
                continue;
            }
        }.path();

        let meta = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                println!("ls {:?}: {}", path, e);
                continue;
            }
        };

        if let Err(e) = deal_with_file(output, &path, &meta, true) {
            println!("ls {:?} {}", path, e);
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse();
    if args.paths.is_empty() {
        args.paths.push(String::from("./"));
    }

    let mut output = Vec::<String>::new();
    for path in args.paths.iter() {
        let metadata = match fs::metadata(path) {
            Ok(md) => md,
            Err(e) => {
                println!("ls {:?}: {}", path, e);
                continue;
            }
        };

        let path = Path::new(path);

        if metadata.is_dir() {
            if let Err(e) = enter_dir(&mut output, path) {
                println!("ls {:?}: {}", path, e);
            }
            continue;
        }

        if let Err(e) = deal_with_file(&mut output, path, &metadata, false) {
            println!("ls {:?} {}", path, e);
        }
    }

    io::stdout().write_all((output.join(" ") + "\n").as_bytes())?;

    Ok(())
}
