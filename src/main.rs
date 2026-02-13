mod compare;
mod dir;
mod terminal;

use anyhow::Result;
use clap::Parser;
use colored::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// First image or directory
    path_a: PathBuf,

    /// Second image or directory
    path_b: PathBuf,

    /// Threshold for difference (0.0 to 1.0)
    #[arg(short, long, default_value_t = 0.1)]
    threshold: f32,

    /// Output path for diff overlay image (single file mode only)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Print preview in terminal
    #[arg(short, long)]
    preview: bool,

    /// Fail if any difference is found (non-zero exit code)
    #[arg(long)]
    fail_on_diff: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.path_a.is_dir() && args.path_b.is_dir() {
        run_dir_diff(&args)
    } else {
        run_file_diff(&args)
    }
}

fn run_file_diff(args: &Args) -> Result<()> {
    let res = compare::compare_images(
        &args.path_a,
        &args.path_b,
        args.threshold,
        args.output.is_some() || args.preview,
    )?;

    println!("{}", "Comparison Result:".bold());
    println!("  Similarity: {:.2}%", res.score * 100.0);
    println!("  Diff Pixels: {}", res.diff_pixels);
    println!("  Total Pixels: {}", res.total_pixels);

    if let Some(diff_img) = &res.diff_image {
        if let Some(output_path) = &args.output {
            diff_img.save(output_path)?;
            println!("  Diff image saved to: {}", output_path.display().to_string().cyan());
        }

        if args.preview {
            println!("\n{}", "Terminal Preview:".bold());
            let dynamic_img = image::DynamicImage::ImageRgba8(diff_img.clone());
            terminal::print_preview(&dynamic_img);
        }
    }

    if args.fail_on_diff && res.diff_pixels > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn run_dir_diff(args: &Args) -> Result<()> {
    let items = dir::compare_directories(&args.path_a, &args.path_b, args.threshold)?;

    println!("\n{:<40} {:<10} {:<10}", "File", "Score", "Status");
    println!("{}", "-".repeat(65));

    let mut diff_count = 0;

    let total_files = items.len();
    for item in items {
        match item.status {
            dir::DirDiffStatus::Match(res) => {
                let status = if res.diff_pixels > 0 {
                    diff_count += 1;
                    "DIFF".red()
                } else {
                    "OK".green()
                };
                println!("{:<40} {:<10.2}% {:<10}", 
                    item.relative_path.display().to_string(),
                    res.score * 100.0,
                    status
                );
            }
            dir::DirDiffStatus::MissingInB => {
                diff_count += 1;
                println!("{:<40} {:<10} {:<10}", 
                    item.relative_path.display().to_string(),
                    "-".dimmed(),
                    "MISSING".yellow()
                );
            }
            dir::DirDiffStatus::Error(e) => {
                println!("{:<40} {:<10} {:<10}", 
                    item.relative_path.display().to_string(),
                    "ERROR".red(),
                    e.yellow()
                );
            }
        }
    }

    println!("\nSummary: {} files compared, {} differences found.", 
        total_files, 
        diff_count
    );

    if args.fail_on_diff && diff_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
