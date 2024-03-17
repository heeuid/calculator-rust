use clap::Parser;
use color_eyre::eyre::Result;
use std::{
    fs::File,
    io::{self, Write, Read},
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
    /// non-printable ascii: 0~8 11~31 127 128~255
    /// 1) ch = 0~8 11~31 127: ^((ch + 1) % 128 + 63)
    ///    e.g. 8 => ^(72) = ^H
    /// 2) ch = 128~159 255: M-^(((ch - 128) + 1) % 128 + 63)
    ///    e.g. 136 => M-^(72) = M-^H
    /// 3) ch = 160~254: M-(ch - 128)
    ///    e.g. 193 => M-A
    #[arg(short = 'v', long, default_value_t = false, verbatim_doc_comment)]
    show_nonprinting: bool,

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

fn replace_non_printables(args: &Args, line: &[u8]) -> String {
    let mut output = String::new();

    for b in line {
        match *b {
            9 if args.tab => output.push_str("^I"),
            10 if args.end => output.push_str("$\n"),
            0..=8 | 11..=31 | 127 => {
                if args.show_nonprinting {
                    output.push('^');
                    output.push(((*b + 1) % 128 + 63) as char);
                }
            }
            128..=255 => {
                if args.show_nonprinting {
                    let byte = *b - 128;
                    output.push_str("M-");
                    match byte {
                        0..=31 | 127 => {
                            if args.show_nonprinting {
                                output.push('^');
                                output.push(((byte + 1) % 128 + 63) as char);
                            }
                        }
                        _ => {
                            output.push(byte as char);
                        }
                    }
                }
            } 
            _ => output.push(*b as char),
        }
    }

    output
}

fn change_form(args: &Args, info: &mut Info, line: &[u8]) -> String {
    let output = replace_non_printables(args, line);

    if args.num {
        let num_str = info.line_num.to_string();
        info.line_num += 1;
        format!("{:>5}  {}", num_str.as_str(), output.as_str())
    } else {
        output
    }
}

fn print_bytes(args: &Args, info: &mut Info, data: &[u8]) -> Result<()> {
    let mut ioout = io::stdout();
    let output = change_form(args, info, data);
    ioout.write_all(output.as_bytes())?;
    Ok(())
}

fn _print_file(args: &Args, info: &mut Info, path: &Path) -> Result<()> {
    let mut data = vec![];
    let file = File::open(path)?;
    io::BufReader::new(file).read_to_end(&mut data)?;
    print_bytes(args, info, &data)?;
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
        print_bytes(args, info, input.as_bytes())?;
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
