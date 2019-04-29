#![feature(process_exitcode_placeholder)]

#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

mod casing;
mod cli;
mod list;
mod logging;

use crate::{
  casing::Casing,
  list::WordList,
};

use lazy_static::lazy_static;

use rand::thread_rng;

use xz2::read::XzDecoder;

use std::{
  io::Write,
  process::ExitCode,
};

const WORD_LIST_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bytes"));

lazy_static! {
  static ref WORD_LISTS: Vec<WordList> = {
    rmp_serde::decode::from_read(XzDecoder::new(WORD_LIST_BYTES)).expect("invalid internal serialization")
  };
}

fn word_list(name: &str) -> Option<&WordList> {
  WORD_LISTS.iter().find(|x| x.short_names.contains(&name.to_string()))
}

fn main() -> ExitCode {
  let args = crate::cli::app(&*WORD_LISTS).get_matches();

  if args.is_present("lists") {
    return lists();
  }

  let list_name = args.value_of("list").expect("list value missing");
  let separator = args.value_of("separator").expect("separator value missing");
  let num_words: u8 = match args.value_of("words").expect("words value missing").parse() {
    Ok(x) if x == 0 => {
      eprintln!("must use at least one word");
      return ExitCode::FAILURE;
    },
    Ok(x) => x,
    Err(e) => {
      eprintln!("invalid amount of words: {}", e);
      return ExitCode::FAILURE;
    },
  };
  let num_pws: usize = match args.value_of("passphrases").expect("passphrases value missing").parse() {
    Ok(x) => x,
    Err(e) => {
      eprintln!("invalid amount of passphrases: {}", e);
      return ExitCode::FAILURE;
    },
  };

  if args.is_present("verbose") {
    crate::logging::set_up();
  }

  let casing = Casing::from_str(args.value_of("casing").expect("casing is a clap default arg"))
    .expect("casing has clap possible_values");

  let list = word_list(list_name).unwrap();
  let mut rng = thread_rng();

  let isnt_tty = atty::isnt(atty::Stream::Stdout);

  let stdout = std::io::stdout();
  let mut lock = stdout.lock();

  for i in 0..num_pws {
    let passphrase = list.generate(&mut rng, num_words, separator, &casing);

    if isnt_tty && i == num_pws - 1 {
      lock.write_all(passphrase.as_bytes()).unwrap();
    } else {
      lock.write_fmt(format_args!("{}\n", passphrase)).unwrap();
    }
  }

  ExitCode::SUCCESS
}

fn lists() -> ExitCode {
  for (i, list) in WORD_LISTS.iter().enumerate() {
    println!("Name: {}", list.name);
    println!("Description: {}", list.description);
    println!("Short names: {}", list.short_names.join(", "));
    println!("Length: {}", list.list.len());

    if i < WORD_LISTS.len() - 1 {
      println!();
    }
  }

  ExitCode::SUCCESS
}
