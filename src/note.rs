use crate::err::Result;

use std::collections::HashSet;
use std::fs;
use std::string::String;

use std::io::Write;
use std::io;
use std::io::BufRead;

use groestl::Digest;

#[derive(Debug, Default, Clone)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub text: String,
    pub tags: HashSet<String>,
}

const TAG_REGEXP: &str = r"[^#]#([[:alnum:]/-_]+)";

impl Note {
    // TODO: use trait for argument
    pub fn from_file(file: &fs::File) -> Result<Note> {
        let buf = io::BufReader::new(file);
        let mut note = Note::default();
        for line_s in buf.lines() {
            let line = line_s.unwrap();
            if line.starts_with("# ") {
                let line = line.trim_matches('#');
                let re = regex::Regex::new(TAG_REGEXP).unwrap();
                let id_cap_opt = re.find(line);
                if id_cap_opt.is_some() {
                    let id_cap = id_cap_opt.unwrap();
                    note.id = line[id_cap.start() + 2..id_cap.end()].to_string();
                    note.title = [line[..id_cap.start()].trim(), line[id_cap.end()..].trim()]
                        .join(" ")
                        .trim()
                        .to_string();
                } else {
                    note.title = line.trim().to_string();
                }
            } else if line.starts_with("[comment]:") {
                continue;
            } else {
                let re = regex::Regex::new(TAG_REGEXP).unwrap();
                note.tags
                    .extend(re.captures_iter(line.as_str()).map(|m| m[1].to_string()));
                note.text.push_str(line.as_str());
                note.text.push('\n');
            }
        }
        return Ok(note);
    }

    // TODO: use trait Write for argument
    pub fn to_file(&self, file: &fs::File) -> Result<()> {
        let mut buf = io::BufWriter::new(file);
        writeln!(&mut buf, "# #{} {}", self.id, self.title).unwrap();
        writeln!(&mut buf, "{}", self.text).unwrap();
        Ok(())
    }

    pub fn templated() -> Result<Note> {
        let mut note = Note::default();
        note.id = "<uid optional>".to_string();
        note.title = "<title>".to_string();
        note.text = r##"
[comment]: # (Use `#` at very beging of 1 line to claim this line title)
[comment]: # (Use `#` to claim next word as a tag, e.g. "This is hi pri #task")"##.to_string();
        Ok(note)
    }

    pub fn gen_uid(&self) -> String {
        let mut hasher = groestl::Groestl224::default();
        hasher.input(self.title.as_str());
        hasher.input(self.text.as_str());
        for tag in self.tags.iter() {
            hasher.input(tag);
        }
        zbase32::encode_full_bytes(
            &hasher.result()
        ).to_string()
    }

    pub fn fix_uid(&mut self)
    {
        let re = regex::Regex::new(TAG_REGEXP).unwrap();
        let mut id_text = " #".to_string();
        id_text.push_str(self.id.as_str());
        if !re.is_match(id_text.as_str()) {
            self.id = self.gen_uid();
        }
    }
}
