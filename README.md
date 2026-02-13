# image-diff

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey)

**image-diff** is a high-performance CLI tool designed for visual regression testing and dataset validation. It provides structural and pixel-level comparison of images with instant terminal-native previews.

## Features

- **Blazing Fast:** Parallel directory processing using Rust's Rayon.
- **Terminal Previews:** View diff heatmaps directly in your terminal using ANSI half-blocks.
- **Dual Metrics:** Reports both raw Pixel Similarity and Structural Similarity (SSIM).
- **Directory Diffing:** Recursively compare folders of images with summary reporting.
- **CI/CD Ready:** Support for JSON output and semantic exit codes.

## Installation

Ensure you have Rust installed, then clone and build:

```bash
git clone https://github.com/cachevector/image-diff
cd image-diff
cargo build --release
```

The binary will be available at `./target/release/image-diff`.

## Usage

### Compare two images
```bash
image-diff baseline.png screenshot.png --preview --output diff.png
```

### Compare directories recursively
```bash
image-diff ./goldens/ ./screenshots/ --threshold 0.1
```

### Automation & CI/CD
Fail the build if any differences are found and output machine-readable results:
```bash
image-diff a.png b.png --json --fail-on-diff
```

### Ignore dynamic regions
Ignore parts of the image that change frequently (like timestamps):
```bash
image-diff a.png b.png --ignore 0,0,100,50 --ignore 500,500,200,100
```

## CLI Options

| Option | Description | Default |
| :--- | :--- | :--- |
| `-t, --threshold` | Sensitivity for pixel comparison (0.0 to 1.0) | `0.1` |
| `-p, --preview` | Render a low-res diff heatmap in the terminal | `false` |
| `-o, --output` | Path to save the high-res diff overlay image | `None` |
| `-i, --ignore` | Ignore region in `x,y,w,h` format | `[]` |
| `--json` | Output machine-readable results in JSON format | `false` |
| `--fail-on-diff` | Return exit code 1 if differences are detected | `false` |
