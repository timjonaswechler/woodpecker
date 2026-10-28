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
