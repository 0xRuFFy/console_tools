use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use colored::Colorize;

const BYTES_IN_KB: u64 = 1024;
const BYTES_IN_MB: u64 = BYTES_IN_KB * 1024;
const BYTES_IN_GB: u64 = BYTES_IN_MB * 1024;

const FILE_SIZE_INDENT: usize = 50;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
#[clap(name = "lsr")]
struct Cli {
    /// Path to directory
    #[clap(value_parser, default_value = ".")]
    location: String,

    /// Recursive depth for listing of sub directories.
    /// Negative value for no limit
    #[clap(short, long, default_value_t = -1, verbatim_doc_comment)]
    depth: i8,

    /// Show hidden files
    #[clap(short, long, action)]
    all: bool,
}

fn get_symbol(i: usize, length: usize, indent: usize) -> &'static str {
    if i == 0 {
        if length == 1 {
            if indent != 0 {
                return "╰";
            }
            return "─";
        } else {
            if indent != 0 {
                return "├";
            }
            return "╭";
        }
    } else if i == length - 1 {
        return "╰";
    };

    "├"
}

fn beautify_bytes(bytes: u64) -> String {
    if bytes < BYTES_IN_KB {
        return format!("{}B", bytes);
    }
    if bytes < BYTES_IN_MB {
        return format!("{:.2}KB", bytes as f64 / 1024.0);
    }
    if bytes < BYTES_IN_GB {
        return format!("{:.2}MB", bytes as f64 / 1024.0 / 1024.0);
    }
    format!("{:.2}GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
}

fn lsr(path: &Path, depth: i8, indent: usize, cli: &Cli) -> Result<u64, String> {
    if !path.is_dir() {
        return Err(format!("{} is not a directory", path.display()));
    }

    if depth == -1 {
        return Ok(0);
    }

    let dir = std::fs::read_dir(path)
        .unwrap()
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap())
        .filter(|r| {
            r.path()
                .file_name()
                .is_some_and(|name| !name.to_str().is_some_and(|s| s.starts_with(".")))
                || cli.all
        });

    let mut total_bytes = 0;
    let length = path.read_dir().unwrap().count();
    for (i, entry) in dir.enumerate() {
        let path = entry.path();
        let bytes = path.metadata().unwrap().len();
        total_bytes += bytes;

        // TODO: Move File/Dir Info to the beginning of each line
        let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
        name = format!(
            "{}{} {}",
            " ".repeat(indent * 2),
            get_symbol(i, length, indent).bright_black(),
            name
        );
        if path.is_dir() {
            println!("{}/", name.bold());
            lsr(&path, depth - 1, indent + 1, cli).unwrap();
        } else {
            println!(
                "{}{}{}",
                name,
                " ".repeat(FILE_SIZE_INDENT - name.len()),
                beautify_bytes(bytes)
            )
        }
    }

    return Ok(total_bytes);
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let path = PathBuf::from(&cli.location);

    if !path.is_dir() {
        eprintln!("{} is not a directory", path.display());
        return ExitCode::FAILURE;
    }

    let mut depth = cli.depth;
    if depth < 0 {
        depth = i8::MAX;
    }

    match lsr(&path, depth, 0, &cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}
