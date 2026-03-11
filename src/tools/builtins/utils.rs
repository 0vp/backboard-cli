use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn get_string(args: &Value, key: &str) -> Option<String> {
    args.get(key)?.as_str().map(|v| v.to_string())
}

pub fn get_usize(args: &Value, key: &str, default: usize) -> usize {
    args.get(key)
        .and_then(Value::as_u64)
        .map(|v| v as usize)
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

pub fn get_optional_usize(args: &Value, key: &str) -> Option<usize> {
    args.get(key).and_then(Value::as_u64).map(|v| v as usize)
}

pub fn resolve_path(workspace_root: &Path, input: Option<&str>) -> Result<PathBuf> {
    let raw = input.unwrap_or(".");
    let path = if Path::new(raw).is_absolute() {
        PathBuf::from(raw)
    } else {
        workspace_root.join(raw)
    };

    let canonical_workspace = workspace_root.canonicalize().with_context(|| {
        format!(
            "cannot canonicalize workspace root {}",
            workspace_root.display()
        )
    })?;
    let canonical_path = path
        .canonicalize()
        .with_context(|| format!("path does not exist or is invalid: {}", path.display()))?;

    if !canonical_path.starts_with(&canonical_workspace) {
        bail!("path {} is outside workspace", canonical_path.display());
    }

    Ok(canonical_path)
}

pub fn paginate<T: Clone>(items: &[T], offset: usize, limit: usize) -> (Vec<T>, bool) {
    if offset >= items.len() {
        return (Vec::new(), false);
    }

    let end = offset.saturating_add(limit).min(items.len());
    let slice = items[offset..end].to_vec();
    let has_more = end < items.len();
    (slice, has_more)
}

#[cfg(test)]
mod tests {
    use super::{paginate, resolve_path};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn paginates_values() {
        let src = vec![1, 2, 3, 4];
        let (subset, has_more) = paginate(&src, 1, 2);
        assert_eq!(subset, vec![2, 3]);
        assert!(has_more);
    }

    #[test]
    fn resolve_path_blocks_parent_escape() {
        let dir = tempdir().expect("tempdir");
        let workspace = dir.path().join("ws");
        fs::create_dir_all(&workspace).expect("create workspace");
        fs::write(workspace.join("a.txt"), "x").expect("write file");

        let workspace_canonical = workspace.canonicalize().expect("canonical workspace");
        let ok = resolve_path(&workspace, Some("a.txt")).expect("resolve inside workspace");
        assert!(ok.starts_with(&workspace_canonical));

        let outside = resolve_path(&workspace, Some("../"));
        assert!(outside.is_err());
    }
}
