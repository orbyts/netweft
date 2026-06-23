use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// One file emitted by an adapter, relative to the adapter output root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub relative_path: PathBuf,
}

/// Collect a stable, sorted manifest of files beneath an output root.
pub fn collect_artifacts(root: &Path) -> Result<Vec<Artifact>> {
    let mut artifacts = Vec::new();
    collect(root, root, &mut artifacts)?;
    artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(artifacts)
}

fn collect(root: &Path, current: &Path, artifacts: &mut Vec<Artifact>) -> Result<()> {
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to inspect {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            collect(root, &path, artifacts)?;
        } else if file_type.is_file() {
            let relative_path = path
                .strip_prefix(root)
                .with_context(|| format!("{} is outside {}", path.display(), root.display()))?
                .to_path_buf();
            artifacts.push(Artifact { relative_path });
        }
    }

    Ok(())
}
