use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

mod cli;

use cli::{Cli, Commands};
use tomet_wiki_assets::ASSETS;
use tomet_wiki_headless::{WikiConfig, build_wiki, run_dev_server};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            dir,
            dest,
            strict,
            production,
        } => {
            let (src_dir, out_dir, mut config) = resolve_build_paths(&dir, dest)?;
            if production {
                config.production = true;
            }

            let report = build_wiki(&src_dir, &out_dir, &config, &ASSETS, false)?;

            if strict && !report.failures.is_empty() {
                for failure in &report.failures {
                    eprintln!("  {}: {}", failure.rel_path, failure.error);
                }
                anyhow::bail!(
                    "{} document(s) failed to build (--strict)",
                    report.failures.len()
                );
            }
        }
        Commands::Serve {
            dir,
            dest,
            host,
            port,
            production,
        } => {
            let (src_dir, out_dir, mut config) = resolve_build_paths(&dir, dest)?;
            if production {
                config.production = true;
            }

            run_dev_server(
                src_dir,
                out_dir,
                config,
                Arc::new(ASSETS),
                host,
                port,
            )
            .await?;
        }
        Commands::Init {
            dir,
            title,
            owner,
            repo,
        } => {
            fs::create_dir_all(&dir)?;
            let target_file = dir.join("wiki.toml");
            if target_file.exists() {
                anyhow::bail!("Configuration already exists: {}", target_file.display());
            }

            let wiki_title = title.unwrap_or_else(|| {
                dir.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("My Wiki")
                    .to_string()
            });
            let cms_owner = owner.unwrap_or_default();
            let cms_repo = repo.unwrap_or_default();

            let template = format!(
                r#"[wiki]
title = "{wiki_title}"
lang = "ja"
dest = "dist"

[cms]
owner = "{cms_owner}"
repo = "{cms_repo}"
branch = "main"

[build]
url_prefix = "/wiki"
exclude = [
  "00-09 System/01 Apps"
]
"#
            );

            fs::write(&target_file, template)?;
            info!("Created configuration: {}", target_file.display());
        }
    }

    Ok(())
}

fn resolve_build_paths(
    dir: &Path,
    dest: Option<PathBuf>,
) -> Result<(PathBuf, PathBuf, WikiConfig)> {
    let config = WikiConfig::load_from_dir(dir)?;
    let src_dir = dir
        .canonicalize()
        .with_context(|| format!("Failed to find source directory {}", dir.display()))?;
    let out_dir = dest.unwrap_or_else(|| src_dir.join(&config.wiki.dest));
    Ok((src_dir, out_dir, config))
}
