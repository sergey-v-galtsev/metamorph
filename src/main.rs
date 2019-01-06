extern crate clap;

use std::path::Path;

mod tissue;

fn search(args: &clap::ArgMatches<>) {
    let notebook = tissue::Notebook::open().unwrap();
    if args.is_present("tag") {
        println!(
            "Tags: #{}",
            args.values_of("tag").unwrap().collect::<Vec<_>>().join(" #")
        );
        let notes = notebook.query(
            &args.values_of("tag").unwrap().collect::<Vec<_>>()
        ).unwrap();
        for (t, n) in notes {
            println!(
                "Note: {} : {} \n    #: {}",
                t,
                n.title,
                n.tags.join(", ")
            );
        }
    }
}

fn note(_args: &clap::ArgMatches<>) {
    let _notebook = tissue::Notebook::open();
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
            clap::SubCommand::with_name("search")
                .about("task tracker")
                .arg(
                    clap::Arg::with_name("tag")
                        .long("tag")
                        .multiple(true)
                        .short("t")
                        .takes_value(true)
                        .help("tag")
                )
                .arg(
                    clap::Arg::with_name("name")
                        .help("name")
                        .long("name")
                        .short("n")
                        .takes_value(true)
                )
                .arg(
                    clap::Arg::with_name("message")
                        .help("message")
                        .long("message")
                        .short("m")
                        .multiple(true)
                        .takes_value(true)
                )
                .arg(
                    clap::Arg::with_name("modify")
                        .long("mod")
                        .short("c")
                        .help("modify existing")
                )
        )
        .subcommand(
            clap::SubCommand::with_name("new")
                .arg(
                    clap::Arg::with_name("name")
                        .long("name")
                        .multiple(true)
                        .short("n")
                        .takes_value(true)
                        .help("tag")
                )
                .arg(
                    clap::Arg::with_name("tag")
                        .long("tag")
                        .multiple(true)
                        .short("t")
                        .takes_value(true)
                        .help("tag")
                )
        )
        .subcommand(
            clap::SubCommand::with_name("name")
                .arg(
                    clap::Arg::with_name("name")
                        .long("name")
                        .multiple(true)
                        .short("n")
                        .takes_value(true)
                        .help("tag")
                )
                .arg(
                    clap::Arg::with_name("tag")
                        .long("tag")
                        .multiple(true)
                        .short("t")
                        .takes_value(true)
                        .help("tag")
                )
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
    println!("SubCommand: {}", args.subcommand_name().unwrap());

    return match args.subcommand() {
        ("search", Some(sub_args)) => {
            search(sub_args)
        },
        ("new", Some(sub_args)) => {
            note(sub_args)
        },
        (_, None) => {
        },
        (_, Some(_)) => {
        },
    }
}
