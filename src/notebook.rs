use crate::err::Error;
use crate::err::Result;
use crate::note::Note;

use std::collections::HashMap;
use std::collections::HashSet;

use std::fs;
use std::path;

use std::io;
use std::io::Write;

use std::string::String;
use std::vec::Vec;

#[derive(Debug)]
pub struct Notebook {
    tag_to_uids: HashMap<String, HashSet<String>>,
    pub notes: HashMap<String, Note>,
    dir_path: path::PathBuf,
}

impl Notebook {
    pub fn on_dir(dir_path: &path::Path) -> Result<Notebook> {
        let mut notebook = Notebook {
            tag_to_uids: HashMap::new(),
            notes: HashMap::new(),
            dir_path: dir_path.clone().to_path_buf(),
        };
        let files = fs::read_dir(dir_path).unwrap();
        for file_r in files {
            let file = file_r.unwrap();
            if file.file_type().unwrap().is_dir() {
                continue;
            }
            let mut note = Note::from_file_on_disk(file.path().as_path()).unwrap();
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

    // TODO: shold be iterator
    pub fn tags(&self) -> Vec<&str> {
        self.tag_to_uids.keys().map(|t| t.as_str()).collect()
    }

    // TODO: shold be iterator
    pub fn uids(&self) -> Vec<&str> {
        self.notes.keys().map(|t| t.as_str()).collect()
    }

    #[allow(dead_code)]
    pub fn search_tags(&self, txt: &str) -> Result<Vec<&str>> {
        let mut tags = Vec::new();
        for tag in self.tag_to_uids.keys() {
            if tag.find(&txt).is_some() {
                tags.push(tag.as_str());
            }
        }
        Ok(tags)
    }

    pub fn expand_tag(&self, tag: &str) -> HashSet<String> {
        self.tag_to_uids.get(tag).map_or(
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

    #[allow(dead_code)]
    pub fn has(&self, tag: &str) -> bool {
        self.notes.contains_key(tag)
    }

    pub fn add_to_index(&mut self, note: Note) -> Result<()> {
        for tag in note.tags.iter() {
            self.tag_to_uids.entry(
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
}

