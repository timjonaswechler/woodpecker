//! Writable application data lives under the existing session artifact directory.
//! Keep it with the session's evidence after exit; never adopt a previous workspace.
use super::Error;
use std::{path::Path, process::Command};

pub(super) fn prepare(
    root: &Path,
    directory: &cap_std::fs::Dir,
    command: &mut Command,
) -> Result<(), Error> {
    directory.create_dir("workspace").map_err(|error| {
        Error::new(
            "invalid_config",
            format!("cannot create a fresh session workspace: {error}"),
        )
    })?;
    let workspace = directory.open_dir("workspace").map_err(Error::io)?;
    for name in ["config", "data", "tmp"] {
        workspace.create_dir(name).map_err(Error::io)?;
    }
    let path = root.join("workspace");
    command.env("WOODPECKER_SESSION_DIR", &path);
    // The launcher and its child inherit the same session-local temporary directory.
    for name in ["TMPDIR", "TMP", "TEMP"] {
        command.env(name, path.join("tmp"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn workspace_is_exclusive_and_sets_only_session_local_paths() {
        let root = std::env::temp_dir().join(format!(
            "woodpecker-workspace-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let directory =
            cap_std::fs::Dir::open_ambient_dir(&root, cap_std::ambient_authority()).unwrap();
        let mut command = Command::new("unused");
        prepare(&root, &directory, &mut command).unwrap();
        for name in ["config", "data", "tmp"] {
            assert!(root.join("workspace").join(name).is_dir());
        }
        let environment: std::collections::HashMap<_, _> = command.get_envs().collect();
        assert_eq!(
            environment[std::ffi::OsStr::new("WOODPECKER_SESSION_DIR")],
            Some(root.join("workspace").as_os_str())
        );
        assert_eq!(
            environment[std::ffi::OsStr::new("TMPDIR")],
            Some(root.join("workspace/tmp").as_os_str())
        );
        std::fs::write(root.join("workspace/config/settings.json"), "preserve").unwrap();
        assert!(prepare(&root, &directory, &mut command).is_err());
        assert_eq!(
            std::fs::read_to_string(root.join("workspace/config/settings.json")).unwrap(),
            "preserve"
        );
        std::fs::remove_dir_all(&root).unwrap();
    }
}
