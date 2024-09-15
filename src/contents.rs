use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::errors::NotAvailableError;

pub struct File {
    pub path: String,
    pub contents: String,
    pub sentences: Vec<String>,
}

impl File {
    pub fn new(path: String, contents: String) -> Self {
        Self {
            path,
            contents,
            sentences: Vec::new(),
        }
    }

    pub fn parse(&mut self) {
        let path = Path::new(&self.path);
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext {
                "rs" => self.parse_rust_file(),
                "md" => self.parse_markdown_file(),
                "toml" => self.parse_toml_file(),
                _ => self.sentences.push(self.contents.clone()),
            }
        } else {
            self.sentences.push(self.contents.clone());
        }
    }

    fn parse_rust_file(&mut self) {
        let mut sentences = Vec::new();
        let mut in_block_comment = false;

        for line in self.contents.lines() {
            let line = line.trim();

            if in_block_comment {
                if let Some(end_pos) = line.find("*/") {
                    let comment = &line[..end_pos];
                    sentences.push(comment.to_string());
                    in_block_comment = false;
                } else {
                    sentences.push(line.to_string());
                }
            } else {
                if let Some(pos) = line.find("///") {
                    // Documentation comment
                    let comment = &line[pos + 3..];
                    sentences.push(comment.trim().to_string());
                } else if let Some(pos) = line.find("//!") {
                    // Inner documentation comment
                    let comment = &line[pos + 3..];
                    sentences.push(comment.trim().to_string());
                } else if let Some(pos) = line.find("//") {
                    // Line comment
                    let comment = &line[pos + 2..];
                    sentences.push(comment.trim().to_string());
                } else if let Some(start_pos) = line.find("/*") {
                    // Start of block comment
                    in_block_comment = true;
                    let after_start = &line[start_pos + 2..];
                    if let Some(end_pos) = after_start.find("*/") {
                        // Block comment ends on the same line
                        let comment = &after_start[..end_pos];
                        sentences.push(comment.to_string());
                        in_block_comment = false;
                    } else {
                        sentences.push(after_start.to_string());
                    }
                }
            }
        }

        self.sentences = sentences;
    }

    fn parse_markdown_file(&mut self) {
        let mut sentences = Vec::new();
        let mut in_code_block = false;

        for line in self.contents.lines() {
            let line = line.trim();

            if line.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            if !in_code_block && !line.starts_with('#') && !line.is_empty() {
                sentences.push(line.to_string());
            }
        }

        self.sentences = sentences;
    }

    fn parse_toml_file(&mut self) {
        let mut sentences = Vec::new();

        for line in self.contents.lines() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // Handle comments starting with '#'
            if line.starts_with('#') {
                let comment = line[1..].trim();
                sentences.push(format!("Comment: {}", comment));
                continue;
            }

            // Handle table headers [table]
            if line.starts_with('[') && line.ends_with(']') {
                let table = &line[1..line.len() - 1].trim();
                sentences.push(format!("Table: {}", table));
                continue;
            }

            // Handle key-value pairs
            if let Some(idx) = line.find('=') {
                let key = line[..idx].trim();
                let value = line[idx + 1..].trim();
                sentences.push(format!("{} = {}", key, value));
                continue;
            }

            // If none of the above, just add the line
            sentences.push(line.to_string());
        }

        self.sentences = sentences;
    }
}

trait HasFileExt {
    fn has_file_extension(&self, endings: &[&str]) -> bool;
}

impl HasFileExt for Path {
    fn has_file_extension(&self, endings: &[&str]) -> bool {
        if let Some(ext) = self.extension().and_then(|e| e.to_str()) {
            endings.contains(&ext)
        } else {
            false
        }
    }
}

// Load files from directory with specified endings
pub fn load_files_from_dir(
    dir: PathBuf,
    endings: &[&str],
    prefix: &PathBuf,
) -> Result<Vec<File>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            let mut sub_files = load_files_from_dir(path, endings, prefix)?;
            files.append(&mut sub_files);
        } else if path.is_file() && path.has_file_extension(endings) {
            println!("Path: {:?}", path);
            let contents = fs::read_to_string(&path)?;
            let path = path.strip_prefix(prefix)?.to_owned();
            let key = path.to_str().ok_or(NotAvailableError {})?;
            let mut file = File::new(key.to_string(), contents);
            file.parse();
            files.push(file);
        }
    }
    Ok(files)
}
