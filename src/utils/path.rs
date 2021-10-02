use std::env::current_exe;
use std::path::PathBuf;

/// Returns the path to the applications resources folder.
///
/// The resources folder should be placed in the same directory
/// as the executable.
pub fn res_path() -> PathBuf {
    // Get exe path
    // See: https://doc.rust-lang.org/std/env/fn.current_exe.html
    // Get parent
    // See: https://stackoverflow.com/a/46749415/12347616
    current_exe().unwrap().parent().unwrap().join("resources")
}

/// Returns the path to the applications attachments folder.
pub fn attachment_path() -> PathBuf {
    current_exe().unwrap().parent().unwrap().join("attachments")
}
