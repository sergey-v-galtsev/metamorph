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

fn new_item(args: &clap::ArgMatches<>) {
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

fn time_log(_args: &clap::ArgMatches<>) {
}

fn test(_args: &clap::ArgMatches<>) {
}

fn main() {
    let args = clap::App::new("tissue")
        .version("0.0.1")
        .author("Alexander Kindyakov <akindyakov@gmail.com>")
        .about("Does awesome things")
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("v")
                .short("v")
                .multiple(true)
                .global(true)
                .help("Sets the level of verbosity")
        )
        .subcommand(
            clap::SubCommand::with_name("new")
                .about("task tracker")
                //.arg(
                //    clap::Arg::with_name("list")
                //        .short("l")
                //        .long("list")
                //        .help("list all existing tasks")
                //)
                .arg(
                    clap::Arg::with_name("tag")
                        .long("tag")
                        .multiple(true)
                        .short("t")
                        .takes_value(true)
                        .help("tag")
                )
                .arg(
                    clap::Arg::with_name("message")
                        .help("message")
                        .long("message")
                        .short("m")
                        .multiple(true)
                        .takes_value(true)
                )
        )
        .subcommand(
            clap::SubCommand::with_name("new")
        )
        .subcommand(
            clap::SubCommand::with_name("test")
                .about("controls testing features")
                .version("1.3")
                .author("Someone E. <someone_else@other.com>")
                .arg(
                    clap::Arg::with_name("debug")
                        .short("d")
                        .help("print debug information verbosely")
                )
        )
        .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config = Path::new(
        args.value_of("config").unwrap_or("~/.tissue.conf")
    );
    println!("Path to config file: {}", config.display());

    return match args.subcommand() {
        ("new", Some(sub_args)) => {
            new_item(sub_args)
        },
        ("test", Some(sub_args)) => {
            test(sub_args)
        },
        ("log", Some(sub_args)) => {
            time_log(sub_args)
        },
        (_, None) => {
        },
        (_, Some(_)) => {
        },
    }
}
