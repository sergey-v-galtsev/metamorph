use crate::err::Error;
use crate::err::Result;
use crate::note::Note;
use crate::notebook::Notebook;

use std::env;
use std::fs;
use std::io;
use std::process;

use std::io::Write;

pub struct InteractiveNotebook {
    notebook: Notebook,
}

pub type INotebook = InteractiveNotebook;

impl InteractiveNotebook {
    pub fn create(nt: Notebook) -> InteractiveNotebook {
        InteractiveNotebook {
            notebook: nt,
        }
    }

    pub fn list(
        &self,
        tags: &Vec<&str>,
        ntags: &Vec<&str>,
    ) -> Result<()> {
        let notes = self.notebook.query_and(&tags, &ntags).unwrap();
        for n in notes {
            println!(
                "#{} {}{}",
                n.id,
                n.title,
                n.tags.iter().fold(
                    String::new(),
                    |mut acc, i| {
                        acc.push_str(" #");
                        acc.push_str(i.as_str());
                        acc
                    }
                ),
            );
        }
        Ok(())
    }

    pub fn show(
        &self,
        tags: &Vec<&str>,
        ntags: &Vec<&str>,
    ) -> Result<()> {
        let notes = self.notebook.query_and(&tags, &ntags).unwrap();
        for n in notes {
            println!(
                "#{} {}{}\n{}",
                n.id,
                n.title,
                n.tags.iter().fold(
                    String::new(),
                    |mut acc, i| {
                        acc.push_str(" #");
                        acc.push_str(i.as_str());
                        acc
                    }
                ),
                n.text,
            );
        }
        Ok(())
    }

    pub fn graph_dot(
        &self,
        tags: &Vec<&str>,
        ntags: &Vec<&str>,
    ) -> Result<()> {
        let notes = self.notebook.query_and(&tags, &ntags).unwrap();
        println!("digraph metamorph {{");
        println!("node [shape=box]");
        for n in notes {
            println!(
                "n_{} [label=\"{}\\n#{}\"]",
                n.id,
                n.title,
                n.id,
            );
            for t in n.tags.iter() {
                println!(
                    "n_{} -> n_{}",
                    t,
                    n.id,
                );
            }
        }
        println!("}}");
        Ok(())
    }

    pub fn iadd(&mut self) -> Result<()> {
        let mut note = self.iedit_note(
            &Note::templated().unwrap()
        ).unwrap();
        if note.text.is_empty() && note.title.is_empty() {
            return Err(Error {
                message: "Empty note file".to_string(),
            });
        }
        note.fix_uid();
        self.notebook.add(note).unwrap();
        Ok(())
    }

    fn write_comments(&self, file: &fs::File) -> Result<()> {
        let mut buf = io::BufWriter::new(file);
        writeln!(&mut buf, "[comment]: # (List of existing tags:)").unwrap();
        for tag in self.notebook.tags() {
            writeln!(&mut buf, "[comment]: # (#{})", tag).unwrap();
        }
        writeln!(&mut buf, "[comment]: # (List of existing notes:)").unwrap();
        for (_, note) in self.notebook.notes.iter() {
            writeln!(&mut buf, "[comment]: # (#{} - {})", note.id, note.title).unwrap();
        }
        Ok(())
    }

    fn iedit_note(
        &self,
        note: &Note
    ) -> Result<Note> {
        let editor = env::var("EDITOR").unwrap_or("/usr/bin/vi".to_string());
        let tmp = tempfile::Builder::new()
            .suffix(".md")
            .rand_bytes(8)
            .tempfile()
            .unwrap();
        note.to_file(tmp.as_file()).unwrap();
        self.write_comments(tmp.as_file()).unwrap();
        let child_status = process::Command::new(editor)
            .arg(tmp.path().as_os_str())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        if !child_status.success() {
            return Err(Error {
                message: "Editor process finished with error".to_string(),
            });
        }
        let new_note = Note::from_file_on_disk(tmp.path()).unwrap();
        Ok(new_note)
    }

    pub fn iedit(
        &mut self,
        tags: &Vec<&str>,
        no_tags: &Vec<&str>,
    ) -> Result<()> {
        let notes = self.notebook.query_and(tags, no_tags).unwrap();
        if notes.len() != 1 {
            return Err(Error {
                message: "To many notes in query result, expected only 1 to iedit".to_string(),
            });
        }
        let new_note = {
            let old_note = &notes[0];
            let mut new_note = self.iedit_note(&old_note).unwrap();
            if new_note.id.is_empty() || new_note.id != old_note.id {
                new_note.id = old_note.id.clone();
            }
            new_note.fix_uid();
            new_note
        };
        self.notebook.add(new_note).unwrap();
        Ok(())
    }
}
