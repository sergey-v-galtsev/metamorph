extern crate chrono;
extern crate clap;
extern crate dirs;
extern crate groestl;
extern crate hex;
extern crate regex;
extern crate tempfile;

use storage::groestl::Digest;
use storage::groestl::Groestl256;

use std::collections::HashMap;
use std::collections::HashSet;

use std::env;
use std::error;
use std::fmt;
use std::fs;
use std::path;
use std::process;
use std::result;

use std::io;
use std::io::BufRead;
use std::io::Seek;
use std::io::Write;

use std::string::String;
use std::vec::Vec;


#[derive(Debug, Clone, Default)]
pub struct Error {
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "invalid first item to double"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

type Result<T> = result::Result<T, Error>;

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
                        .to_string();
                } else {
                    note.title = line.to_string();
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
        write!(&mut buf, "# #{} {}\n\n", self.id, self.title).unwrap();
        if !self.tags.is_empty() {
            write!(
                &mut buf,
                " {}\n\n",
                self.tags
                    .iter()
                    .fold("#".to_string(), |mut acc: String, item| {
                        acc.push_str(", #");
                        acc.push_str(item);
                        acc
                    })
            )
            .unwrap();
        }
        writeln!(&mut buf, "{}", self.text).unwrap();
        Ok(())
    }

    pub fn gen_uid(&self) -> String {
        let mut hasher = Groestl256::default();
        hasher.input(self.title.as_str());
        hasher.input(self.text.as_str());
        for tag in self.tags.iter() {
            hasher.input(tag);
        }
        hex::encode(hasher.result()).to_string()
    }
}

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
            notebook.add(note).unwrap();
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

    pub fn query(&self, tags: &Vec<&str>) -> Result<HashMap<String, Note>> {
        let mut ret = HashMap::new();
        for tag in tags {
            let group = self.expand_tag(tag);
            for tag in group {
                ret.insert(
                    tag.to_string(),
                    self.notes.get(tag.as_str()).unwrap().clone(),
                );
            }
        }
        Ok(ret)
    }

    pub fn has(&self, tag: &str) -> bool {
        self.notes.contains_key(tag)
    }

    pub fn add(&mut self, note: Note) -> Result<()> {
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

    fn make_note_template(&self, file: &fs::File) -> Result<()> {
        let mut buf = io::BufWriter::new(file);
        write!(&mut buf, "# \n\n\n").unwrap();
        writeln!(
            &mut buf,
            "[comment]: # (Use `#` at very beging of 1 line to claim this line title)"
        )
        .unwrap();
        writeln!(
            &mut buf,
            "[comment]: # (Use `#` to claim next word as a tag, e.g. \"This is hi pri #task\")"
        )
        .unwrap();
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

    pub fn iadd(&mut self) -> Result<()> {
        let editor = env::var("EDITOR").unwrap_or("/usr/bin/vi".to_string());
        println!("Editor: {}", editor);
        let tmp = tempfile::Builder::new()
            .suffix(".md")
            .rand_bytes(8)
            .tempfile()
            .unwrap();
        println!("Tmp file: {}", tmp.path().display());
        self.make_note_template(tmp.as_file()).unwrap();
        tmp.as_file().sync_all().unwrap();
        tmp.as_file().seek(io::SeekFrom::Start(0)).unwrap();
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
        let mut note = Note::from_file(tmp.as_file()).unwrap();
        if note.text.is_empty() && note.title.is_empty() {
            return Err(Error {
                message: "Empty note file".to_string(),
            });
        }
        if note.id.is_empty() {
            note.id = self.notes.len().to_string();
        }
        if self.has(note.id.as_str()) {
            note.id.push('_');
            let uid = note.gen_uid();
            note.id.push_str(uid.as_str());
        }
        let task_path = self.dir_path.join([note.id.as_str(), "md"].join("."));
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(task_path)
            .unwrap();
        note.to_file(&file).unwrap();
        self.add(note).unwrap();
        Ok(())
    }
}
