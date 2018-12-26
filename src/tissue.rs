extern crate chrono;
extern crate clap;
extern crate groestl;
extern crate hex;
extern crate kv;

use groestl::{
    Digest,
    Groestl256,
};
use std::fs::OpenOptions;
use std::io::{
    BufWriter,
    Write,
};
use std::path::Path;

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

pub fn time_log(_args: &clap::ArgMatches<>) {
}

pub fn test(_args: &clap::ArgMatches<>) {
}


#[derive(Debug, Clone)]
pub struct Error;

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

pub struct Tissue {
    _cfg: Config,
}

impl Tissue {
    pub fn open(cfg: Config) -> Result<Tissue> {
        Ok(Tissue{_cfg : cfg})
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    smth: i32,
}

impl Config {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            smth: 1
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

    pub fn build(self) -> Config {
        self.t
    }
}
