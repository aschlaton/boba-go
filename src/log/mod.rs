use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// path for log files
fn get_temp_dir() -> PathBuf {
    let temp_dir = PathBuf::from("temp");
    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir).ok();
    }
    temp_dir
}

pub fn debug(category: &str, msg: impl AsRef<str>) {
    let temp_dir = get_temp_dir();
    let filename = temp_dir.join(format!("{}_debug.log", category));
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
    {
        writeln!(file, "{}", msg.as_ref()).ok();
    }
}

pub fn host(msg: impl AsRef<str>) {
    debug("host", msg);
}

pub fn client(msg: impl AsRef<str>) {
    debug("client", msg);
}
