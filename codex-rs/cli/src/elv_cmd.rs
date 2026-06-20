use anyhow::Context;
use clap::Args;
use std::env;
use std::path::PathBuf;
use tokio::process::Command;

/// Run ELV learner tools through the local elv_core JSON CLI.
#[derive(Debug, Args)]
pub struct ElvCommand {
    /// Arguments forwarded to `python -m elv_core.cli`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

pub async fn run(cmd: ElvCommand) -> anyhow::Result<()> {
    let python = env::var("ELV_CORE_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let mut child = Command::new(python);
    child.arg("-m").arg("elv_core.cli").args(cmd.args);

    if let Some(src_path) = adjacent_elv_core_src() {
        let merged = match env::var_os("PYTHONPATH") {
            Some(existing) if !existing.is_empty() => {
                let mut paths = vec![src_path];
                paths.extend(env::split_paths(&existing));
                env::join_paths(paths).context("failed to build PYTHONPATH for elv_core")?
            }
            _ => env::join_paths([src_path]).context("failed to build PYTHONPATH for elv_core")?,
        };
        child.env("PYTHONPATH", merged);
    }

    let status = child
        .status()
        .await
        .context("failed to run python -m elv_core.cli")?;

    if !status.success() {
        anyhow::bail!("python -m elv_core.cli failed with status {status}");
    }

    Ok(())
}

fn adjacent_elv_core_src() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    let candidate = cwd.parent()?.join("EmbodiedLearningVolition").join("src");
    candidate.exists().then_some(candidate)
}
