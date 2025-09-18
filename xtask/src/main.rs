use std::{
    env, fs,
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
    cmd !(sh, "cargo install --locked --git https://github.com/getzola/zola").run()?;

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
