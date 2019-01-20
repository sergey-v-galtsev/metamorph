extern crate clap;
extern crate dirs;

use std::option::Option;
use std::path::Path;
use std::string::String;

mod storage;

fn spawn_notebook(args: &clap::ArgMatches) -> Option<storage::Notebook> {
    if args.is_present("on-dir") {
        return Some(
            storage::Notebook::on_dir(Path::new(args.value_of("on-dir").unwrap())).unwrap(),
        );
    }
    None
}

fn list(notebook: &mut storage::Notebook, args: &clap::ArgMatches) {
    if args.is_present("tag") {
        println!(
            "Tags: #{}",
            args.values_of("tag")
                .unwrap()
                .collect::<Vec<_>>()
                .join(" #")
        );
        let notes = notebook
            .query(&args.values_of("tag").unwrap().collect::<Vec<_>>())
            .unwrap();
        for (t, n) in notes {
            println!(
                "#{}: {}\n    {}\n",
                t,
                n.title,
                n.tags
                    .iter()
                    .fold("#".to_string(), |mut acc: String, item| {
                        acc.push_str(item);
                        acc.push_str(", #");
                        acc
                    }),
            );
        }
    }
}

fn show(notebook: &mut storage::Notebook, args: &clap::ArgMatches) {
    let tag = args.value_of("tag").unwrap();
    let notes = notebook.query(&vec![tag]).unwrap();
    let (_, note) = notes.iter().next().unwrap();
    println!("#{} {}\n", note.id, note.title);
    println!(
        "{}",
        note.tags
            .iter()
            .fold(" #".to_string(), |mut acc: String, item| {
                acc.push_str(item);
                acc.push_str(" #");
                acc
            })
    );
    println!("{}\n", note.text);
}

fn note(notebook: &mut storage::Notebook, _args: &clap::ArgMatches) {
    notebook.iadd().unwrap();
}

fn main() {
    let args = clap::App::new("metamorph: notebook")
        .version("0.0.1")
        .arg(
            clap::Arg::with_name("on-dir")
                .long("on-dir")
                .value_name("DIR")
                .help("Path to notebook directory")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("v")
                .short("v")
                .multiple(true)
                .global(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            clap::SubCommand::with_name("list")
                .about("list")
                .arg(
                    clap::Arg::with_name("tag")
                        .long("tag")
                        .multiple(true)
                        .short("t")
                        .takes_value(true)
                        .help("tag"),
                )
                .arg(
                    clap::Arg::with_name("name")
                        .help("name")
                        .long("name")
                        .short("n")
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::with_name("message")
                        .help("message")
                        .long("message")
                        .short("m")
                        .multiple(true)
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::with_name("modify")
                        .long("mod")
                        .short("c")
                        .help("modify existing"),
                ),
        )
        .subcommand(clap::SubCommand::with_name("new"))
        .subcommand(
            clap::SubCommand::with_name("show").arg(
                clap::Arg::with_name("tag")
                    .long("tag")
                    .short("t")
                    .takes_value(true)
                    .help("tag"),
            ),
        )
        .get_matches();

    let mut notebook = spawn_notebook(&args).unwrap();
    return match args.subcommand() {
        ("list", Some(sub_args)) => list(&mut notebook, sub_args),
        ("new", Some(sub_args)) => note(&mut notebook, sub_args),
        ("show", Some(sub_args)) => show(&mut notebook, sub_args),
        (_, None) => {}
        (_, Some(_)) => {}
    };
}
