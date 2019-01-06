extern crate chrono;
extern crate clap;
extern crate groestl;
extern crate hex;
extern crate kv;
extern crate tempfile;

use groestl::{
    Digest,
    Groestl256,
};
use std::fs::OpenOptions;
use std::io::{
    BufReader,
    BufWriter,
    Write,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::string::String;
use std::vec::Vec;
use std::io::BufRead;


#[allow(dead_code)]
pub fn new_item(args: &clap::ArgMatches<>) {
    // TODO: take values from config
    let todo_root_path = Path::new(
        "/home/akindyakov/source/git.note/todo"
    );
    let rel_todo_list_path = Path::new(
        "list.md"
    );
    let rel_tasks_dir_path = Path::new(
        "files"
    );
    let todo_list_path = todo_root_path.join(rel_todo_list_path);
    if args.is_present("message") {
        println!(
            "Message: {}",
            args.values_of("message").unwrap().collect::<Vec<_>>().join(" ")
        );
    }
    if args.is_present("tag") {
        println!(
            "Tags: #{}",
            args.values_of("tag").unwrap().collect::<Vec<_>>().join(" #")
        );
    }
    if args.is_present("list") {
        println!("List of tasks");
        return;
    }
    let mut hasher = Groestl256::default();
    let message = args
        .values_of("message")
        .unwrap()
        .collect::<Vec<_>>()
        .join(" ");
    hasher.input(&message);
    let tags = args.values_of("tag").unwrap().collect::<Vec<_>>();
    for tag in tags.iter() {
        hasher.input(tag);
    }
    let uid = hex::encode(hasher.result());
    let rel_task_file_path = rel_tasks_dir_path.join(
        [uid.as_str(), "md"].join(".")
    );
    let task_file_path = todo_root_path.join(&rel_task_file_path);
    {
        println!("Open {}", todo_list_path.display());
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(todo_list_path).unwrap();
        let mut buf = BufWriter::new(&file);
        writeln!(&mut buf,
                " - [{}]({}) {}",
                &uid,
                rel_task_file_path.display(),
                &message
        ).unwrap();
    }
    println!("Create new {} {} {}", &uid, &message, rel_task_file_path.display());
    {
        let time_format = "%Y.%m.%d %u %H:%M";
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(task_file_path).unwrap();
        let mut buf = BufWriter::new(&file);
        write!(&mut buf, "# {} {}\n\n", &uid, &message).unwrap();
        write!(&mut buf, " #{}\n\n", tags.join(" #")).unwrap();
        write!(&mut buf, "[back]({})\n", rel_todo_list_path.display()).unwrap();
        write!(
            &mut buf,
            "Created: {}\n\n",
            chrono::Local::now().format(&time_format)
        ).unwrap();
    }
}

#[derive(Debug, Clone, Default)]
pub struct Error {
    message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "invalid first item to double"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default, Clone)]
pub struct Note {
    pub title: String,
    pub text: String,
    pub tags: Vec<String>,
}

impl Note {
    // TODO: use trait for argument
    pub fn from_file(file: &std::fs::File) -> Result<Note> {
        let buf = BufReader::new(file);
        let mut note = Note::default();
        for line_s in buf.lines() {
            let line = line_s.unwrap();
            if line.starts_with("# ") {
                note.title = line
                    .trim()
                    .trim_start_matches('#')
                    .to_string();
            }
            note.text.push_str(line.as_str());
            note.tags.extend(
                line.split(" #")
                    .skip(1)
                    .map(
                        |t| t.split_whitespace().next().unwrap().to_string()
                    )
            );
        }
        return Ok(note);
    }

    // TODO: use trait Write for argument
    pub fn to_file(&self, file: &std::fs::File) -> Result<()> {
        let mut buf = BufWriter::new(file);
        // TODO
        Ok(())
    }

    pub fn gen_uid(&self) -> Result<String> {
        let mut uid = self.title.to_lowercase();
        uid.retain(|ch| ch.is_alphanumeric());
        uid.truncate(8);
        uid.push('_');
        let mut hasher = Groestl256::default();
        hasher.input(self.text);
        for tag in self.tags.iter() {
            hasher.input(tag);
        }
        uid.push_str(
            hex::encode(hasher.result())
        );
    }
}

// impl Default for Note {
//     fn default() -> Self {
//         Self {
//         }
//     }
// }

#[derive(Debug, Default)]
pub struct Notebook  {
    tag_to_tags: HashMap<String, Vec<String>>,
    notes: HashMap<String, Note>,
}

fn read_note_from_file(path: &Path) -> Result<Note> {
    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .unwrap();
    return Note::from_file(&file);
}

impl Notebook {
    pub fn open() -> Result<Notebook> {
        let dir_path = Path::new("/home/akindyakov/source/git.note/todo/files");
        println!("Path to config file: {}", dir_path.display());
        let mut notebook = Notebook::default();
        let files = fs::read_dir(dir_path).unwrap();
        for file_r in files {
            let file = file_r.unwrap();
            let mut note = read_note_from_file(file.path().as_path()).unwrap();
            let uniq_tag = file.path().file_stem().unwrap().to_str().unwrap().to_string();
            note.tags.push(uniq_tag.clone());
            for tag in &note.tags {
                let tag_refs = notebook.tag_to_tags.entry(tag.to_string()).or_insert(
                    Vec::new()
                );
                tag_refs.push(uniq_tag.clone());
            }
            notebook.notes.insert(
                uniq_tag,
                note
            );
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

    pub fn expand_tag(&self, tag: &str) -> Vec<String> {
        self.tag_to_tags.get(tag).map_or(
            [tag.to_string()].to_vec(),
            |g| g.clone()
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

    #[allow(dead_code)]
    pub fn add(&self, _note: &Note) -> Result<()> {
        Err(Error::default())
    }

    pub fn iadd(&self) -> Result<()> { // -> Result<&str> {
        let editor = std::env::var("EDITOR")
            .unwrap_or(
                "/usr/bin/vi".to_string()
            );
        println!("Editor: {}", editor);
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        println!("Tmp file: {}", tmp.path().display());
        // let mut editor_child = std::process::Command::new(editor)
        //     .arg(tmp.path().as_os_str())
        //     .spawn()
        //     .expect("d");
        // let editor_output = editor_child.wait().unwrap();
        let child_status = std::process::Command::new(editor)
            .arg(tmp.path().as_os_str())
            .spawn()
            .unwrap().wait().unwrap();
        if ! child_status.success() {
            return Err(Error{message: "Editor process finished with error".to_string()});
        }
        let note = Note::from_file(tmp.as_file());
        // TODO: interactively read tags and title if required
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    //path: Path,
}

#[allow(dead_code)]
impl Config {
    #[allow(dead_code)]
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
        }
    }
}

#[derive(Debug, Default)]
pub struct Builder {
    t: Config,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            t: Config::default(),
        }
    }

    #[allow(dead_code)]
    pub fn build(self) -> Config {
        self.t
    }
}
