#![allow(clippy::collapsible_else_if)]

use std::error::Error;

use camino::Utf8PathBuf;
use clap::Parser;
use glob::glob;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    auto_execute: bool,

    #[arg(long, short)]
    quiet: bool,

    #[arg(long, short, default_values = ["jpg", "png", "gif", "webp", "webm", "mp4", "avif", "mkv", "avi"])]
    formats: Vec<String>,

    search_root: Utf8PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let glob_expression = format!("{}/**/*", args.search_root);

    let search_results: Vec<(Utf8PathBuf, Utf8PathBuf)> = glob(&glob_expression)?
        .filter_map(Result::ok)
        .map(Utf8PathBuf::from_path_buf)
        .filter_map(Result::ok)
        .filter_map(|path| {
            if let Ok(Some(inferred_type)) = infer::get_from_path(&path) {
                let current_extension = path.extension().unwrap_or("");
                let inferred_extension = inferred_type.extension();

                if (current_extension != inferred_extension)
                    && (args.formats.contains(&inferred_extension.to_owned()))
                {
                    let fixed_path = path.with_extension(inferred_extension);
                    Some((path, fixed_path))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    for (path, fixed_path) in search_results {
        if fixed_path.try_exists()? {
            let hash_a = sha256::try_digest(&path)?;
            let hash_b = sha256::try_digest(&fixed_path)?;

            if hash_a == hash_b {
                if !args.quiet {
                    println!("Deduplicating '{}' and '{}'", path, fixed_path);
                }

                if args.auto_execute {
                    std::fs::remove_file(path)?;
                }
            } else {
                if !args.quiet {
                    println!(
                        "Cannot deduplicate '{}' and '{}', hash mismatch",
                        path, fixed_path
                    );
                }
            }
        } else {
            if !args.quiet {
                println!("Renaming '{}' to '{}'", path, fixed_path);
            }

            if args.auto_execute {
                std::fs::rename(path, fixed_path)?;
            }
        }
    }

    Ok(())
}
