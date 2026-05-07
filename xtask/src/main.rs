use std::{
    env, ffi, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use xshell::{cmd, Shell};

// -------------------------------------------------------------
// Helper function that define various "settings"
// -------------------------------------------------------------

fn source_dir() -> Result<PathBuf> {
    Path::new("site/")
        .canonicalize()
        .context("site source directory does not exist")
}

/// Get a list of source directories for all wasm modules
fn wasm_source_dirs() -> Result<Vec<PathBuf>> {
    let entries = Path::new("wasm/")
        .canonicalize()
        .map(|p| p.read_dir())
        .flatten()
        .context("could not read 'wasm' dir")?;
    let subdirs = entries
        .filter_map(|res| res.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    Ok(subdirs)
}

/// Get path of site's static directory
fn static_dir() -> Result<PathBuf> {
    source_dir().map(|path| path.join("static"))
}

fn build_dir() -> Result<PathBuf> {
    env::current_dir()
        .map(|p| p.join("public"))
        .context("failed to construct path of build directory")
}

// -------------------------------------------------------------
// Command line interface
// -------------------------------------------------------------

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch the theme
    Setup,
    /// Build the site to `./public`
    Build,
    /// Serve the site locally for development
    Serve,
    /// Build and update wasm modules
    Wasm,
    /// Clean build artifacts
    Clean,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    match cli.command {
        Commands::Setup => setup(&sh),
        Commands::Build => build(&sh),
        Commands::Serve => serve(&sh),
        Commands::Wasm => wasm(&sh),
        Commands::Clean => clean(&sh),
    }
}

// -------------------------------------------------------------
// Subcommand implementations
// -------------------------------------------------------------

fn setup(sh: &Shell) -> Result<()> {
    // Init submodules
    cmd!(sh, "git submodule update --init").run()?;

    // Install zola
    cmd!(
        sh,
        "cargo install --locked --git https://github.com/getzola/zola"
    )
    .run()?;

    // Install wasm-pack
    cmd!(sh, "cargo install --locked wasm-pack").run()?;

    // Return success
    Ok(())
}

fn serve(sh: &Shell) -> Result<()> {
    let source_dir = source_dir()?;

    // Serve site
    sh.change_dir(source_dir);
    cmd!(sh, "zola serve").run()?;

    // Return success
    Ok(())
}

fn build(sh: &Shell) -> Result<()> {
    let source_dir = source_dir()?;
    let output_dir = build_dir()?;

    // Remove old builds
    clean(sh)?;

    // First, build the site
    sh.change_dir(source_dir);
    cmd!(sh, "zola build --output-dir {output_dir}").run()?;

    // Return success
    Ok(())
}

fn wasm(sh: &Shell) -> Result<()> {
    let static_dir = static_dir()?;
    for dir in wasm_source_dirs()? {
        // Build wasm
        sh.change_dir(&dir);
        cmd!(sh, "wasm-pack build --target web --out-dir pkg").run()?;
        // Determine which output files to copy
        let output_dir = dir.join("pkg").canonicalize()?;
        let copy_files = output_dir
            .read_dir()?
            .filter_map(|r| r.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| {
                let ext = p.extension().and_then(|s| s.to_str());
                match ext {
                    Some("js") => true,
                    Some("wasm") => true,
                    Some(_) => false,
                    None => false,
                }
            })
            .collect::<Vec<PathBuf>>();
        // Copy these output files to the blog's static folder
        for file in copy_files {
            let dest_path = static_dir.join(file.file_name().unwrap());
            fs::copy(&file, &dest_path).context(format!(
                "Failed to copy {} to {}",
                file.display(),
                dest_path.display()
            ))?;
        }
    }
    Ok(())
}

fn clean(_: &Shell) -> Result<()> {
    let output_dir = build_dir()?;

    // Remove output directory
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)
            .context(format!("could not remove {}", output_dir.display()))?;
    }

    // Return success
    Ok(())
}
