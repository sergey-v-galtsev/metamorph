use crate::err::Error;
use crate::err::Result;
use crate::note::Note;

use std::collections::HashMap;
use std::collections::HashSet;

use std::env;
use std::fs;
use std::path;
use std::process;

use std::io;
use std::io::Write;

use std::string::String;
use std::vec::Vec;

#[derive(Debug)]
pub struct Notebook {
    tag_to_tags: HashMap<String, HashSet<String>>,
    notes: HashMap<String, Note>,
    dir_path: path::PathBuf,
}

fn read_note_from_file(path: &path::Path) -> Result<Note> {
    let file = fs::OpenOptions::new().read(true).open(path).unwrap();
    return Note::from_file(&file);
}

impl Notebook {
    pub fn on_dir(dir_path: &path::Path) -> Result<Notebook> {
        let mut notebook = Notebook {
            tag_to_tags: HashMap::new(),
            notes: HashMap::new(),
            dir_path: dir_path.clone().to_path_buf(),
        };
        let files = fs::read_dir(dir_path).unwrap();
        for file_r in files {
            let file = file_r.unwrap();
            if file.file_type().unwrap().is_dir() {
                continue;
            }
            let mut note = read_note_from_file(file.path().as_path()).unwrap();
            if note.id.is_empty() {
                note.id = file
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
            }
            notebook.add_to_index(note).unwrap();
        }
        Ok(notebook)
    }

    #[allow(dead_code)]
    pub fn search_tags(&self, txt: &str) -> Result<Vec<&str>> {
        let mut tags = Vec::new();
        for tag in self.tag_to_tags.keys() {
            if tag.find(&txt).is_some() {
                tags.push(tag.as_str());
            }
        }
        Ok(tags)
    }

    pub fn expand_tag(&self, tag: &str) -> HashSet<String> {
        self.tag_to_tags.get(tag).map_or(
            HashSet::from([tag.to_string()].iter().cloned().collect()),
            |g| g.clone(),
        )
    }

    pub fn expand_tag_as_and(
        &self,
        incl_tags: &Vec<&str>,
        not_incl_tags: &Vec<&str>,
    ) -> HashSet<String> {
        let mut tags = HashSet::new();
        if incl_tags.is_empty() {
            for uid in self.notes.keys() {
                tags.insert(uid.clone());
            }
        } else {
            for group in incl_tags.iter().map(|t| self.expand_tag(t)) {
                tags.extend(group.clone());
            }
        }
        for group in not_incl_tags.iter().map(|t| self.expand_tag(t)) {
            tags = tags.difference(&group).cloned().collect();
        }
        return tags;
    }

    pub fn query_and(
        &self,
        tags: &Vec<&str>,
        no_tags: &Vec<&str>,
    ) -> Result<Vec<Note>> {
        let mut ret = Vec::new();
        for uid in self.expand_tag_as_and(tags, no_tags) {
            let note = self.notes.get(uid.as_str());
            if note.is_some() {
                ret.push(
                    note.unwrap().clone(), // TODO: take note as a ref
                );
            }
        }
        ret.dedup_by(|first, second| first.id == second.id);
        Ok(ret)
    }

    pub fn iedit(
        &mut self,
        tags: &Vec<&str>,
        no_tags: &Vec<&str>,
    ) -> Result<()> {
        let tags = self.expand_tag_as_and(tags, no_tags);
        if tags.len() != 1 {
            return Err(Error {
                message: "To many notes in query result, expected only 1 to iedit".to_string(),
            });
        }
        let new_note = {
            let old_note = self.notes.get(
                tags.iter().next().unwrap().as_str()
            ).unwrap();
            let mut new_note = self.iedit_note(&old_note).unwrap();
            if new_note.id.is_empty() || new_note.id != old_note.id {
                new_note.id = old_note.id.clone();
            }
            new_note.fix_uid();
            new_note
        };
        self.add(new_note).unwrap();
        Ok(())
    }

    #[allow(dead_code)]
    pub fn has(&self, tag: &str) -> bool {
        self.notes.contains_key(tag)
    }

    pub fn add_to_index(&mut self, note: Note) -> Result<()> {
        for tag in note.tags.iter() {
            self.tag_to_tags.entry(
                tag.to_string()
            ).or_insert(HashSet::new()).insert(
                note.id.clone()
            );
        }
        self.notes.insert(note.id.clone(), note);
        Ok(())
    }

    pub fn add(&mut self, note: Note) -> Result<()> {
        let task_path = self.dir_path.join([note.id.as_str(), "md"].join("."));
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(task_path)
            .unwrap();
        note.to_file(&file).unwrap();
        self.add_to_index(note).unwrap();
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
        self.add(note).unwrap();
        Ok(())
    }

    fn write_comments(&self, file: &fs::File) -> Result<()> {
        let mut buf = io::BufWriter::new(file);
        writeln!(&mut buf, "[comment]: # (List of existing tags:)").unwrap();
        for (tag, _) in self.tag_to_tags.iter() {
            writeln!(&mut buf, "[comment]: # (#{})", tag).unwrap();
        }
        writeln!(&mut buf, "[comment]: # (List of existing notes:)").unwrap();
        for (_, note) in self.notes.iter() {
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
        let new_note = read_note_from_file(tmp.path()).unwrap();
        Ok(new_note)
    }
}

