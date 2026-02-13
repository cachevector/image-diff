use crate::compare::{compare_images, DiffResult, Region};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
pub enum DirDiffStatus {
    Match(DiffResult),
    MissingInB,
    Error(String),
}

#[derive(Serialize)]
pub struct DirDiffItem {
    pub relative_path: PathBuf,
    pub status: DirDiffStatus,
}

pub fn compare_directories(
    dir_a: &Path,
    dir_b: &Path,
    threshold: f32,
    ignore_regions: &[Region],
    mask_path: Option<&Path>,
) -> Result<Vec<DirDiffItem>> {
    let files_a: Vec<PathBuf> = WalkDir::new(dir_a)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_image(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();

    let pb = ProgressBar::new(files_a.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap());

    let results: Vec<DirDiffItem> = files_a
        .into_par_iter()
        .map(|path_a| {
            let relative = path_a.strip_prefix(dir_a).unwrap();
            let path_b = dir_b.join(relative);

            let status = if !path_b.exists() {
                DirDiffStatus::MissingInB
            } else {
                match compare_images(&path_a, &path_b, threshold, false, ignore_regions, mask_path) {
                    Ok(res) => DirDiffStatus::Match(res),
                    Err(e) => DirDiffStatus::Error(e.to_string()),
                }
            };

            pb.inc(1);

            DirDiffItem {
                relative_path: relative.to_path_buf(),
                status,
            }
        })
        .collect();

    pb.finish_with_message("Done");
    Ok(results)
}

fn is_image(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp")
}
