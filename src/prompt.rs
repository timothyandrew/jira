use std::error::Error;
use tempfile::NamedTempFile;
use std::process::Command;
use std::fs;
use std::env;
use std::io::Write;

pub async fn text_from_editor(template: &str) -> Result<Option<(String, String)>, Box<dyn Error>> {
    let editor = env::var("EDITOR").unwrap_or("nano".to_owned());

    let temp_file = NamedTempFile::new()?;
    temp_file.as_file().write_all(template.as_bytes()).expect("Failed to write template to the temp file.");

    let temp_path = temp_file.into_temp_path();

    let canonical_path = fs::canonicalize(&temp_path).unwrap();
    Command::new(editor).arg(&canonical_path).status().expect("Failed to open $EDITOR");

    let contents = fs::read_to_string(&temp_path).expect(&format!("Failed to read file: {:?}", temp_path));
    let mut contents = contents.trim().split("\n");
    let (first_line, other_lines) = (contents.next(), contents.collect::<Vec<_>>().join("\n"));

    temp_path.close()?;

    Ok(match (first_line, other_lines) {
        (None, _) => None,
        (Some(""), _) => None,
        (Some(first_line), other_lines) => Some((first_line.trim().to_owned(), other_lines.trim().to_owned()))
    })
}