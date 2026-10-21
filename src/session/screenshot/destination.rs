use super::Diagnostic;
use cap_std::fs::{Dir, OpenOptions};
use std::{
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

pub(super) fn failed(error: impl std::fmt::Display) -> Diagnostic {
    Diagnostic::new("screenshot_failed", error)
}

fn path_error(error: io::Error) -> Diagnostic {
    if error.kind() == io::ErrorKind::PermissionDenied {
        Diagnostic::new("invalid_screenshot_path", error)
    } else {
        failed(error)
    }
}

/// All filesystem operations are relative to this directory capability, including
/// symlink resolution. Re-check at write time, not just when accepting the command.
pub(super) fn prepare(root: &Dir, path: &str) -> Result<bool, Diagnostic> {
    super::validate(path)?;
    let parent = Path::new(path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty());
    if let Some(parent) = parent {
        match root.open_dir(parent) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                root.create_dir_all(parent).map_err(path_error)?;
                root.open_dir(parent).map_err(path_error)?;
            }
            Err(error) => return Err(path_error(error)),
        }
    }
    match root.metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(failed("screenshot destination is not a regular file")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            // A dangling in-root link is still an existing entry to replace.
            Ok(root.symlink_metadata(path).is_ok())
        }
        Err(error) => Err(path_error(error)),
    }
}

pub(super) fn write(root: &Dir, path: &str, png: &[u8]) -> Result<bool, Diagnostic> {
    let overwritten = prepare(root, path)?;
    let path = Path::new(path);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let directory = root.open_dir(parent).map_err(path_error)?;
    let name = path.file_name().expect("validated path");
    let (temporary, mut file) = loop {
        let temporary = format!(
            ".woodpecker-{}-{}.tmp",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        );
        match directory.open_with(&temporary, OpenOptions::new().write(true).create_new(true)) {
            Ok(file) => break (temporary, file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(failed(error)),
        }
    };
    let result = (|| {
        file.write_all(png).map_err(failed)?;
        file.sync_all().map_err(failed)?;
        directory
            .rename(&temporary, &directory, name)
            .map_err(failed)?;
        Ok(overwritten)
    })();
    if result.is_err() {
        let _ = directory.remove_file(&temporary);
    }
    result
}
