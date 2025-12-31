use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let cmd = args
        .next()
        .ok_or_else(|| anyhow!("missing command (try: `cargo run -p xtask -- roadmap-dags`)"))?;

    match cmd.as_str() {
        "roadmap-dags" => {
            let mut pass = vec!["scripts/update_roadmap_dags.py".to_string()];
            pass.extend(args);
            run("python3", &pass)?;
        }
        "install-githooks" => {
            // This sets a local repo config (not global). It's the simplest way to enable
            // version-controlled hooks in `.githooks/`.
            run("git", &["config".into(), "core.hooksPath".into(), ".githooks".into()])?;
        }
        other => {
            bail!("unknown xtask command: {other}");
        }
    }

    Ok(())
}

fn run(bin: &str, args: &[String]) -> Result<()> {
    let status = Command::new(bin)
        .args(args)
        .status()
        .with_context(|| format!("failed to run `{bin}`"))?;

    if !status.success() {
        bail!("command failed: `{bin} {}`", args.join(" "));
    }
    Ok(())
}

