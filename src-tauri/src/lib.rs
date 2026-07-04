// This crate is built as a library target because Cargo.toml declares [lib]
// for Tauri packaging compatibility. The desktop runtime entry remains in
// src-tauri/src/main.rs.

pub fn library_target_ready() -> bool {
    true
}
