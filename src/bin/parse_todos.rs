#![allow(unused_must_use)]
use std::{fs::{self, File, OpenOptions}, path::Path, process::Command};
use std::io::Write;

fn main() {
    let mut dest = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("todo.txt")
        .expect("Could not open todo.txt");

    parse_todos(Path::new("./src"), &mut dest);
}

fn parse_todos(dir: &Path, dest: &mut File) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.file_name().is_some_and(|name| name == "check_todos.rs") {
            continue;
        }

        if path.is_dir() {
            parse_todos(&path, dest);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let content = fs::read_to_string(&path).unwrap();
            let mut lines = content.lines().enumerate().peekable();

            while let Some((i, line)) = lines.next() {
                if line.contains("TODO") {
                    writeln!(dest, "===========================================");
                    writeln!(dest, "{}:{}", path.display(), i + 1);
                    writeln!(dest, "===========================================");
                    writeln!(dest, "{}", line.trim());

                    while let Some(&(_, next_line)) = lines.peek() {
                        let trimmed = next_line.trim();

                        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                            let (j, actual_line) = lines.next().unwrap();
                                writeln!(dest, "{}", actual_line.trim());
                        } else {
                            break;
                        }
                    }

                    writeln!(dest)
                        .expect("Failed to write to todo.txt");
                }
            }
        }
    }
}
