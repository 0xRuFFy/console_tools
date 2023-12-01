use clap::Parser;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const BYTES_IN_KB: f64 = 1024.0;
const BYTES_IN_MB: f64 = BYTES_IN_KB * 1024.0;
const BYTES_IN_GB: f64 = BYTES_IN_MB * 1024.0;

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
    let bytes = bytes as f64;
    if bytes < BYTES_IN_KB {
        return format!("{}B", bytes);
    }
    if bytes < BYTES_IN_MB {
        return format!("{:.2}KB", bytes / BYTES_IN_KB);
    }
    if bytes < BYTES_IN_GB {
        return format!("{:.2}MB", bytes / BYTES_IN_MB);
    }
    format!("{:.2}GB", bytes / BYTES_IN_GB)
}

fn lsr(path: &Path, depth: i8, indent: usize, cli: &Cli) -> Result<u64, String> {
    if !path.is_dir() {
        return Err(format!("{} is not a directory", path.display()));
    }

    if depth == -1 {
        return Ok(0);
    }

    let dir = match std::fs::read_dir(path) {
        Ok(dir) => dir
            .into_iter()
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .filter(|r| {
                r.path()
                    .file_name()
                    .is_some_and(|name| !name.to_str().is_some_and(|s| s.starts_with(".")))
                    || cli.all
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let mut total_bytes: u64 = 0;
    let length = dir.len();
    for (i, entry) in dir.iter().enumerate() {
        let path = entry.path();
        let bytes = match path.metadata() {
            Ok(m) => m.len(),
            Err(e) => {
                return Err(e.to_string());
            }
        };
        total_bytes += bytes;

        // TODO: Move File/Dir Info to the beginning of each line
        let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
        name = format!(
            "{}{} {}",
            " ".repeat(indent * 2),
            get_symbol(i, length, indent).dimmed(),
            name
        );
        if path.is_dir() {
            println!("{}/", if depth == 0 { name.normal() } else { name.dimmed() });
            if let Err(e) = lsr(&path, depth - 1, indent + 1, cli) {
                println!("{}{} {}", " ".repeat((indent + 1) * 2), "╰".dimmed(), e);
            }
        } else {
            println!("{}  {}", name, beautify_bytes(bytes))
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
