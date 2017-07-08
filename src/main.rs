#![deny(missing_docs)]
//! A tool for creating roller shutter images, which emulate how a phone's rolling shutter sees.

extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate image;
extern crate regex;

use clap::{Arg, ArgMatches, App};

mod errors;
mod file_processing;
mod image_processing;

use self::errors::{ErrorKind, Result, ResultExt};
use self::file_processing::*;

/// The *starting* direction of the shutter. That is, what part of the image does the shutter start
/// from, and then go to the other side.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Direction {
    N,
    E,
    S,
    W,
}

impl<'a> From<&'a str> for Direction {
    fn from(s: &'a str) -> Direction {
        match s {
            "N" => Direction::N,
            "E" => Direction::E,
            "S" => Direction::S,
            "W" => Direction::W,
            _ => unreachable!(),
        }
    }
}

quick_main!(run);

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("Rolling Shutter")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Creates a rolling shutter simulation of a set of frames.")
        .arg(Arg::with_name("direction")
            .short("d")
            .long("direction")
            .help("Changes direction of shutter movement; specifically, it determines the \
                   cardinal direction where the shutter *starts* from.")
            .takes_value(true)
            .possible_values(&["N", "E", "S", "W"])
            .default_value("N"))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .help("Output filename.")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("input")
            .short("i")
            .long("input")
            .help("File mask for input.{n}Supported syntax is only for sequential inputs of the \
                   form %3d or %03d. Examples: f%3d.png, foo%03d.jpg")
            .takes_value(true)
            .conflicts_with("folder")
            .index(1))
        .arg(Arg::with_name("folder")
            .short("f")
            .long("folder")
            .help("A folder to use for frames.{n}Frames will be taken in platform-sorted order.")
            .takes_value(true)
            .required_unless("input"))
        .arg(Arg::with_name("quiet")
            .short("q")
            .long("quiet")
            .help("Suppress output."))
        .get_matches()
}

fn run() -> Result<()> {
    let matches = parse_args();

    let direction = matches.value_of("direction").unwrap().into();

    let path_mode = if let Some(path) = matches.value_of("folder") {
        PathMode::Folder(path)
    } else if let Some(path) = matches.value_of("input") {
        PathMode::FileMask(path)
    } else {
        unreachable!();
    };

    let output = matches.value_of("output").unwrap();

    let paths = file_processing::get_paths(&path_mode).chain_err(|| ErrorKind::CouldNotGetPaths)?;

    image_processing::process_images(paths, &output, direction, matches.is_present("quiet"))?;

    Ok(())
}
