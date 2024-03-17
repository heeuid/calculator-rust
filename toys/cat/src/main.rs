use clap::Parser;
use color_eyre::eyre::Result;
use std::{
    fs::File,
    io::{self, BufRead, Write},
    path::Path,
};

#[derive(Parser, Debug)]
#[command(version, about = None, long_about =
    "Concatenate FILE(s) to standard output.\n\
    Copy standard input to standard output when there's no FILE(s) input."
)]
struct Args {
    /// Print TAB to ^I
    #[arg(short, long, default_value_t = false, verbatim_doc_comment)]
    tab: bool,

    /// Print LINEFEED to '$' + LINEFEED
    #[arg(short, long, default_value_t = false, verbatim_doc_comment)]
    end: bool,

    /// Non-printable ascii to printable ascii except for TAB & LINEFEED
    /// e.g. \r to ^M
    #[arg(short, long, default_value_t = false, verbatim_doc_comment)]
    verbal: bool,

    /// Print line numbers
    #[arg(short, long, default_value_t = false, verbatim_doc_comment)]
    num: bool,

    /// File paths
    /// If [FILES] is empty, copy standard input to standard output
    #[clap(value_parser, required = false, verbatim_doc_comment)]
    files: Vec<String>,
}

struct Info {
    line_num: u32,
}

fn replace_non_printables(args: &Args, line: &str) -> String {
    let mut output = String::new();

    for b in line.as_bytes() {
        match b {
            0..=8 | 11..=31 => {
                output.push('^');
                output.push((b + 65) as char);
            }
            9 if args.tab => output.push_str("^I"),
            10 if args.end => output.push_str("$\n"),
            127 => output.push_str("^?"),
            _ => output.push(*b as char),
        }
    }

    output
}

fn change_form(args: &Args, info: &mut Info, line: &str) -> String {
    let output = replace_non_printables(args, line);

    if args.num {
        let num_str = info.line_num.to_string();
        info.line_num += 1;
        format!("{:>5}  {}", num_str.as_str(), output.as_str())
    } else {
        output
    }
}

fn print_line(args: &Args, info: &mut Info, line: &str) -> Result<()> {
    let mut ioout = io::stdout();
    let output = change_form(args, info, line);
    ioout.write_all(output.as_bytes())?;
    Ok(())
}

fn _print_file(args: &Args, info: &mut Info, path: &Path) -> Result<()> {
    let file = File::open(path)?;
    for line in io::BufReader::new(file).lines() {
        print_line(args, info, &line?)?;
    }
    Ok(())
}

fn print_file(args: &Args, info: &mut Info, path: &Path) {
    if let Err(e) = _print_file(args, info, path) {
        println!("cat {:?} {}", path, e);
    }
}

fn _print_stdin(args: &Args, info: &mut Info) -> Result<()> {
    let mut input = String::new();
    let ioin = io::stdin();
    loop {
        ioin.read_line(&mut input)?;
        print_line(args, info, input.as_str())?;
        input.clear();
    }
}

fn print_stdin(args: &Args, info: &mut Info) {
    if let Err(e) = _print_stdin(args, info) {
        println!("cat stdin {}", e);
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let mut info = Info { line_num: 1 };

    if args.files.is_empty() {
        print_stdin(&args, &mut info);
    } else {
        for file in &args.files {
            print_file(&args, &mut info, Path::new(file.as_str()));
        }
    }

    Ok(())
}
