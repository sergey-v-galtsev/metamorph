extern crate clap;

use std::path::Path;

mod tissue;

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

    let _tis = tissue::Tissue::open(
        tissue::Config::builder().build()
    );

    return match args.subcommand() {
        ("new", Some(sub_args)) => {
            tissue::new_item(sub_args)
        },
        ("test", Some(sub_args)) => {
            tissue::test(sub_args)
        },
        ("log", Some(sub_args)) => {
            tissue::time_log(sub_args)
        },
        (_, None) => {
        },
        (_, Some(_)) => {
        },
    }
}
