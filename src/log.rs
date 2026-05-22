use crate::utils;
use std::io::Write;

pub fn info(message: &str) {
  let line = format!(
    "[{}][INFO] {}",
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
    message
  );
  println!("[croak]{}", line);
  write_log_to_file(&line);
}

pub fn warn(message: &str) {
  let line = format!(
    "[{}][WARN] {}",
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
    message
  );
  println!("[croak]{}", line);
  write_log_to_file(&line);
}

pub fn error(message: &str) {
  let line = format!(
    "[{}][ERROR] {}",
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
    message
  );
  eprintln!("[croak]{}", line);
  write_log_to_file(&line);
}

pub fn debug(message: &str) {
  let line = format!(
    "[{}][DEBUG] {}",
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
    message
  );
  write_log_to_file(&line);
}

fn write_log_to_file(line: &str) {
  match get_log_file() {
    Ok(mut file) => {
      if let Err(e) = writeln!(file, "{}", line) {
        eprintln!(
          "[croak][{}][ERROR] Failed to write to log file: {}",
          chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
          e
        );
      }
    },
    Err(e) => {
      eprintln!(
        "[croak][{}][ERROR] Failed to open log file for writing: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        e
      );
      eprintln!();
    },
  }
}

fn get_log_file() -> std::io::Result<std::fs::File> {
  let log_path = utils::get_data_dir().join(utils::LOG_FILE.clone());
  std::fs::create_dir_all(log_path.parent().unwrap())?;
  std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(log_path)
}
