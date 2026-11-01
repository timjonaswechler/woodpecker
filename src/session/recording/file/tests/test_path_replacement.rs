use super::*;
use std::{io::Read, os::unix::fs::symlink, sync::mpsc, time::Duration};

#[test]
fn recording_and_replay_pin_parents_and_reject_swapped_final_entries() {
    let path = crate::test_support::artifacts("recording-path-swap");
    let inside = path.join("root");
    let outside = path.join("outside");
    std::fs::create_dir_all(&inside).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("protected.jsonl"), b"outside sentinel").unwrap();
    let root = Dir::open_ambient_dir(&inside, cap_std::ambient_authority()).unwrap();
    // Pause the real writer between opening its parent and create_new.
    std::fs::create_dir(inside.join("writer")).unwrap();
    let recording = Recording::start_at(
        &root,
        "writer/protected.jsonl".into(),
        &Clock::Application {},
        |file| file,
        || {
            std::fs::rename(inside.join("writer"), inside.join("writer-retired")).unwrap();
            symlink(&outside, inside.join("writer")).unwrap();
        },
    )
    .unwrap();
    assert_eq!(recording.finish("stopped").unwrap(), 0);
    assert_eq!(
        crate::test_support::json_lines(inside.join("writer-retired/protected.jsonl")),
        vec![
            json!({"type":"recording_started","format_version":1}),
            json!({"type":"recording_ended","outcome":"stopped","recorded_commands":0})
        ]
    );
    assert_eq!(
        std::fs::read(outside.join("protected.jsonl")).unwrap(),
        b"outside sentinel"
    );

    // The loader independently pins the already-open parent, not its replacement.
    std::fs::create_dir(inside.join("reader")).unwrap();
    std::fs::write(
        inside.join("reader/protected.jsonl"),
        b"original regular file",
    )
    .unwrap();
    let mut file = open_read_at(&root, "reader/protected.jsonl", || {
        std::fs::rename(inside.join("reader"), inside.join("reader-retired")).unwrap();
        symlink(&outside, inside.join("reader")).unwrap();
    })
    .unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"original regular file");
    // Final-entry races must be checked separately by both actual accessors.
    for mode in ["symlink", "existing"] {
        let dirname = format!("write-{mode}");
        std::fs::create_dir(inside.join(&dirname)).unwrap();
        let dest = inside.join(&dirname).join("protected.jsonl");
        let result = Recording::start_at(
            &root,
            format!("{dirname}/protected.jsonl"),
            &Clock::Application {},
            |f| f,
            || {
                if mode == "symlink" {
                    symlink(outside.join("protected.jsonl"), &dest).unwrap();
                } else {
                    std::fs::write(&dest, b"existing regular sentinel").unwrap();
                }
            },
        );
        let error = match result {
            Err(e) => e,
            Ok(_) => panic!("Writer accepted swapped existing target"),
        };
        assert_eq!(
            error.code(),
            if mode == "symlink" {
                "invalid_recording_path"
            } else {
                "recording_path_exists"
            }
        );
        assert_eq!(
            std::fs::read(outside.join("protected.jsonl")).unwrap(),
            b"outside sentinel"
        );
        if mode == "existing" {
            assert_eq!(std::fs::read(dest).unwrap(), b"existing regular sentinel");
        }
    }
    std::fs::create_dir(inside.join("read-final")).unwrap();
    std::fs::write(inside.join("read-final/protected.jsonl"), b"original").unwrap();
    let error = open_read_at(&root, "read-final/protected.jsonl", || {
        std::fs::remove_file(inside.join("read-final/protected.jsonl")).unwrap();
        symlink(
            outside.join("protected.jsonl"),
            inside.join("read-final/protected.jsonl"),
        )
        .unwrap();
    })
    .unwrap_err();
    assert_eq!(error.code(), "invalid_recording_path");

    // A swapped FIFO must not block in open. On regression, open an owned rescue
    // endpoint solely to unblock/join the worker, then fail rather than leaking it.
    std::fs::write(inside.join("fifo.jsonl"), b"before FIFO").unwrap();
    let directory = root.try_clone().unwrap();
    let fifo = inside.join("fifo.jsonl");
    let worker_fifo = fifo.clone();
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = open_read_at(&directory, "fifo.jsonl", || {
            std::fs::remove_file(&worker_fifo).unwrap();
            let name = std::ffi::CString::new(worker_fifo.as_os_str().as_encoded_bytes()).unwrap();
            assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        })
        .map(|_| ())
        .map_err(|e| e.code().to_owned());
        sender.send(result).unwrap();
    });
    let result = receiver.recv_timeout(Duration::from_secs(2));
    if result.is_err() {
        let rescue = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&fifo)
            .unwrap();
        worker.join().unwrap();
        drop(rescue);
        panic!("Replay open blocked on a non-regular FIFO");
    }
    worker.join().unwrap();
    assert_eq!(result.unwrap().unwrap_err(), "invalid_recording_path");
    assert_eq!(
        std::fs::read(outside.join("protected.jsonl")).unwrap(),
        b"outside sentinel"
    );
    assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 1);
}
