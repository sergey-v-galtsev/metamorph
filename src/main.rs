extern crate clap;
extern crate dirs;

use std::option::Option;
use std::path::Path;
use std::string::String;
use std::vec::Vec;

mod storage;

fn spawn_notebook(args: &clap::ArgMatches) -> Option<storage::Notebook> {
    if args.is_present("on-dir") {
        return Some(
            storage::Notebook::on_dir(
                Path::new(
                    args.value_of("on-dir").unwrap()
                )
            ).unwrap(),
        );
    }
    None
}

fn list(notebook: &mut storage::Notebook, args: &clap::ArgMatches) {
    let mut tags = Vec::new();
    if let Some(os) = args.values_of("tag") {
        tags.extend(os);
    }
    let mut ntags = Vec::new();
    if let Some(os) = args.values_of("ntag") {
        ntags.extend(os);
    }
    println!("Include all tags from [{}]", tags.join(", "));
    println!("Exclude all tags from [{}]", ntags.join(", "));
    let notes = notebook.query_and(&tags, &ntags).unwrap();
    for (_, n) in notes {
        println!(
            "#{}: {}\n    [{}]\n",
            n.id,
            n.title,
            n.tags.iter().fold(
                String::new(),
                |mut acc, i| {
                    acc.push('#');
                    acc.push_str(i.as_str());
                    acc.push_str(", ");
                    acc
                }
            ),
        );
    }
}

fn show(notebook: &mut storage::Notebook, args: &clap::ArgMatches) {
    let mut tags = Vec::new();
    if let Some(os) = args.values_of("tag") {
        tags.extend(os);
    }
    let mut ntags = Vec::new();
    if let Some(os) = args.values_of("ntag") {
        ntags.extend(os);
    }
    println!("Include all tags from [{}]", tags.join(", "));
    println!("Exclude all tags from [{}]", ntags.join(", "));
    let notes = notebook.query_and(&tags, &ntags).unwrap();
    for (_, n) in notes {
        println!(
            "#{}: {}\n    [{}]\n{}",
            n.id,
            n.title,
            n.tags.iter().fold(
                String::new(),
                |mut acc, i| {
                    acc.push('#');
                    acc.push_str(i.as_str());
                    acc.push_str(", ");
                    acc
                }
            ),
            n.text,
        );
    }
}

fn note(notebook: &mut storage::Notebook, _args: &clap::ArgMatches) {
    notebook.iadd().unwrap();
}

fn main() {
    let args = clap::App::new("metamorph: notebook")
        .version("0.0.0")
        .arg(
            clap::Arg::with_name("on-dir")
                .long("on-dir")
                .value_name("DIR")
                .help("Path to notebook directory")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("tag")
                .long("tag")
                .multiple(true)
                .short("t")
                .takes_value(true)
                .global(true)
                .help("tag"),
        )
        .arg(
            clap::Arg::with_name("ntag")
                .long("ntag")
                .multiple(true)
                .takes_value(true)
                .global(true)
                .help("Not include this tag"),
        )
        .subcommand(
            clap::SubCommand::with_name("list")
                .about("list")
        )
        .subcommand(
            clap::SubCommand::with_name("new")
                .about("new")
        )
        .subcommand(
            clap::SubCommand::with_name("show")
                .about("show")
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
