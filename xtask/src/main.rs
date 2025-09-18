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

fn deploy_dir() -> Result<PathBuf> {
    env::current_dir()
        .map(|p| p.join("public"))
        .context("failed to construct deployment path")
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
    /// Serve the site locally for development
    Serve,
    /// Deploy the site to `./public`
    Deploy,
    /// Clean build artifacts
    Clean,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    match cli.command {
        Commands::Setup => setup(&sh),
        Commands::Serve => serve(&sh),
        Commands::Deploy => depoy(&sh),
        Commands::Clean => clean(&sh),
    }
}

// -------------------------------------------------------------
// Subcommand implementations
// -------------------------------------------------------------

fn setup(sh: &Shell) -> Result<()> {
    // Init submodules
    cmd!(sh, "git submodule update --init").run()?;

    // Return success
    Ok(())
}

fn serve(sh: &Shell) -> Result<()> {
    let source_dir = source_dir()?;

    // Run project setup
    setup(sh)?;

    // Serve site
    sh.change_dir(source_dir);
    cmd!(sh, "zola serve").run()?;

    // Return success
    Ok(())
}

fn depoy(sh: &Shell) -> Result<()> {
    let source_dir = source_dir()?;
    let deploy_dir = deploy_dir()?;

    // Remove old deployments
    clean(sh)?;

    // First, build the site
    sh.change_dir(source_dir);
    cmd!(sh, "zola build --output-dir {deploy_dir}").run()?;

    // Return success
    Ok(())
}

fn clean(_: &Shell) -> Result<()> {
    let deploy_dir = deploy_dir()?;

    // Remove deployment directory
    if deploy_dir.exists() {
        fs::remove_dir_all(&deploy_dir)
            .context(format!("could not remove {}", deploy_dir.display()))?;
    }

    // Return success
    Ok(())
}
