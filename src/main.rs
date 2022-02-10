//! This program started as a fork of the Norad "glif_print" example:
//! https://github.com/linebender/norad/blob/master/examples/glif_print.rs
//!
//! You pass this a glyph, and it tries to load it. It will then write it
//! back out to xml, and print this to stdout; you can redirect this to a file
//! in order to inspect how a given glyph would be serialized.
//!
//! Afterwards it will print the xml tree to stderr, which may be useful when
//! debugging parse errors.

//use std::any::type_name;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::{env, fs, io};

use failure::Error;
use quick_xml::{
    events::{attributes::Attribute, Event},
    Reader,
};

use norad::Glyph;
use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;

fn print_type_of<T>(_: &T) {
    println!("TYPE: {}", std::any::type_name::<T>())
}

fn main() -> Result<(), io::Error> {
    let path = match env::args().nth(1).map(PathBuf::from) {
        Some(ref p) if p.exists() && p.extension() == Some(OsStr::new("glif")) => p.to_owned(),
        Some(ref p) => {
            eprintln!("path {:?} is not an existing .glif file, exiting", p);
            std::process::exit(1);
        }
        None => {
            eprintln!("Please supply a path to a glif file");
            std::process::exit(1);
        }
    };

    let glyph = Glyph::load(&path).unwrap();
    println!("DEBUG: {:?}", glyph);
    print_type_of(&glyph);
    print_type_of(&glyph.name);
    println!("{}", glyph.name);
    let to_xml = glyph.encode_xml().unwrap();
    let to_xml = String::from_utf8(to_xml).unwrap();
    // redirect this to a file to get the rewritten glif
    println!("{}", to_xml);


    let svg_data = Data::new()
        .move_to((10, 10))
        .line_by((0, 50))
        .line_by((50, 0))
        .line_by((0, -50))
        .close();

    let svg_path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("d", svg_data);

    let svg_document = Document::new()
        .set("viewBox", (0, 0, 2048, 2048))
        .add(svg_path);

    svg::save("on-chain-nft-image.svg", &svg_document).unwrap();

    let xml = fs::read_to_string(&path)?;
    match print_tokens(&xml) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("error {}", e);
            std::process::exit(1);
        }
    }
}

fn print_tokens(xml: &str) -> Result<(), Error> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    reader.trim_text(true);
    let mut level = 0;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Decl(decl)) => {
                let version = decl.version()?;
                let version = std::str::from_utf8(&version)?;

                let encoding = decl.encoding().transpose()?.unwrap_or_default();
                let encoding = std::str::from_utf8(&encoding)?;

                eprintln!("xml version {} encoding {}", version, encoding);
            }
            Ok(Event::Start(start)) => {
                let name = std::str::from_utf8(start.name())?;
                eprint!("{}<{}", spaces_for_level(level), name);
                for attr in start.attributes() {
                    let attr = attr?;
                    let key = std::str::from_utf8(attr.key)?;
                    let value = attr.unescaped_value()?;
                    let value = reader.decode(&value)?;
                    eprint!(" {}=\"{}\"", key, value);
                }
                eprintln!(">");
                level += 1;
            }
            Ok(Event::End(end)) => {
                level -= 1;
                let name = std::str::from_utf8(end.name())?;
                eprintln!("{}</{}>", spaces_for_level(level), name);
            }
            Ok(Event::Empty(start)) => {
                let name = std::str::from_utf8(start.name())?;
                eprint!("{}<{}", spaces_for_level(level), name);
                for attr in start.attributes() {
                    let Attribute { key, value } = attr?;
                    let key = std::str::from_utf8(key)?;
                    let value = std::str::from_utf8(&value)?;
                    eprint!(" {}=\"{}\"", key, value);
                }
                eprintln!("/>");
            }
            Ok(Event::Eof) => break,
            Ok(other) => eprintln!("{:?}", other),
            Err(e) => {
                eprintln!("error {:?}", e);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

fn spaces_for_level(level: usize) -> &'static str {
    let spaces = "                                                                                                                                                  ";
    let n_spaces = (level * 2).min(spaces.len());
    &spaces[..n_spaces]
}
