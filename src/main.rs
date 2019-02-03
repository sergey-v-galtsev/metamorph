mod err;
mod note;
mod notebook;

extern crate chrono;
extern crate clap;
extern crate dirs;
extern crate groestl;
extern crate regex;
extern crate tempfile;
extern crate zbase32;

use std::option::Option;
use std::path::Path;
use std::string::String;
use std::vec::Vec;

fn spawn_notebook(args: &clap::ArgMatches) -> Option<notebook::Notebook> {
    if args.is_present("on-dir") {
        return Some(
            notebook::Notebook::on_dir(
                Path::new(
                    args.value_of("on-dir").unwrap()
                )
            ).unwrap(),
        );
    }
    None
}

fn list(notebook: &mut notebook::Notebook, args: &clap::ArgMatches) {
    let mut tags = Vec::new();
    if let Some(os) = args.values_of("tag") {
        tags.extend(os);
    }
    let mut ntags = Vec::new();
    if let Some(os) = args.values_of("ntag") {
        ntags.extend(os);
    }
    println!("Include tags [{}]", tags.join(", "));
    println!("Exclude tags [{}]", ntags.join(", "));
    let notes = notebook.query_and(&tags, &ntags).unwrap();
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
}

fn show(notebook: &mut notebook::Notebook, args: &clap::ArgMatches) {
    let mut tags = Vec::new();
    if let Some(os) = args.values_of("tag") {
        tags.extend(os);
    }
    let mut ntags = Vec::new();
    if let Some(os) = args.values_of("ntag") {
        ntags.extend(os);
    }
    println!("Include tags [{}]", tags.join(", "));
    println!("Exclude tags [{}]", ntags.join(", "));
    let notes = notebook.query_and(&tags, &ntags).unwrap();
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
}

fn note(notebook: &mut notebook::Notebook, _args: &clap::ArgMatches) {
    notebook.iadd().unwrap();
}

fn graph(notebook: &mut notebook::Notebook, args: &clap::ArgMatches) {
    let mut tags = Vec::new();
    if let Some(os) = args.values_of("tag") {
        tags.extend(os);
    }
    let mut ntags = Vec::new();
    if let Some(os) = args.values_of("ntag") {
        ntags.extend(os);
    }
    let notes = notebook.query_and(&tags, &ntags).unwrap();
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
}

fn edit(notebook: &mut notebook::Notebook, args: &clap::ArgMatches) {
    let mut tags = Vec::new();
    if let Some(os) = args.values_of("tag") {
        tags.extend(os);
    }
    let mut ntags = Vec::new();
    if let Some(os) = args.values_of("ntag") {
        ntags.extend(os);
    }
    println!("Include tags [{}]", tags.join(", "));
    println!("Exclude tags [{}]", ntags.join(", "));
    notebook.iedit(&tags, &ntags).unwrap();
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
        .subcommand(
            clap::SubCommand::with_name("edit")
                .about("edit")
        )
        .subcommand(
            clap::SubCommand::with_name("graph")
                .about("graph")
                .arg(
                    clap::Arg::with_name("format")
                        .long("format")
                        .takes_value(true)
                        .default_value("dot")
                        .possible_value("dot")
                        .help("Format of ouput graph")
                )
        )
        .get_matches();

    let mut notebook = spawn_notebook(&args).unwrap();
    return match args.subcommand() {
        ("list", Some(sub_args)) => list(&mut notebook, sub_args),
        ("new", Some(sub_args)) => note(&mut notebook, sub_args),
        ("show", Some(sub_args)) => show(&mut notebook, sub_args),
        ("edit", Some(sub_args)) => edit(&mut notebook, sub_args),
        ("graph", Some(sub_args)) => graph(&mut notebook, sub_args),
        (_, None) => {}
        (_, Some(_)) => {}
    };
}
