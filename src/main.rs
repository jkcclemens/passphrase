#![feature(process_exitcode_placeholder)]

#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

mod cli;
mod list;
mod logging;

use crate::list::WordList;

use lazy_static::lazy_static;

use rand::thread_rng;

use xz2::read::XzDecoder;

use std::{
  borrow::Cow,
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

/// Casing for output.
///
/// Note that this has no effect on separators. This only applies to casing conventions. The
/// `CamelCase` enum member will cause the first letter of the first word to be lowercase, then the
/// first letter of all remaining words to be uppercase, regardless of separator.
///
/// This means that choosing a separator of `"-"` and a casing of `CamelCase` will result in
/// `this-Is-An-Example`.
pub enum Casing {
  /// Do not specially format output.
  Standard,
  /// Format output in lowercase.
  Lowercase,
  /// Format output in UPPERCASE.
  Uppercase,
  /// Format output in PascalCase.
  PascalCase,
  /// Format output in camelCase.
  CamelCase,
}

impl Casing {
  fn from_str(s: &str) -> Option<Self> {
    let c = match s {
      "standard" => Casing::Standard,
      "lowercase" => Casing::Lowercase,
      "uppercase" => Casing::Uppercase,
      "pascalcase" => Casing::PascalCase,
      "camelcase" => Casing::CamelCase,
      _ => return None,
    };

    Some(c)
  }

  /// Apply casing to a single word.
  ///
  /// Takes the string slice containing just the word and the index of the word, starting from `0`.
  fn apply_word<'s>(&self, s: &'s str, n: usize) -> Cow<'s, str> {
    match *self {
      Casing::Standard => Cow::Borrowed(s),
      Casing::Lowercase if Casing::all_lower(s) => Cow::Borrowed(s),
      Casing::Lowercase => Cow::Owned(s.to_lowercase()),
      Casing::Uppercase if Casing::all_upper(s) => Cow::Borrowed(s),
      Casing::Uppercase => Cow::Owned(s.to_uppercase()),
      Casing::PascalCase => Casing::case_word(s, None),
      Casing::CamelCase => Casing::case_word(s, Some(n)),
    }
  }

  fn all_lower(s: &str) -> bool {
    s.chars().all(|c| c.is_lowercase())
  }

  fn all_upper(s: &str) -> bool {
    s.chars().all(|c| c.is_uppercase())
  }

  /// Specify `n` as `Some` word index to indicate that this is camelCase.
  fn case_word(s: &str, n: Option<usize>) -> Cow<str> {
    let should_be_lower = n.map(|n| n == 0).unwrap_or(false);
    if should_be_lower && Casing::all_lower(s) {
      return Cow::Borrowed(s);
    }
    let mut chars = s.chars();
    let first = chars.next();
    if !should_be_lower && first.map(|c| c.is_uppercase()).unwrap_or(false) && Casing::all_lower(&s[1..]) {
      return Cow::Borrowed(s);
    }
    let mut res = String::with_capacity(s.len());
    if let Some(c) = first {
      if should_be_lower {
        for c in c.to_lowercase() {
          res.push(c);
        }
      } else {
        for c in c.to_uppercase() {
          res.push(c);
        }
      }
    }
    for c in chars {
      for c in c.to_lowercase() {
        res.push(c);
      }
    }
    Cow::Owned(res)
  }
}

#[cfg(test)]
mod test {
  use super::Casing;

  use std::borrow::Cow;

  #[test]
  fn standard() {
    let casing = Casing::Standard;
    assert_eq!(Cow::Borrowed("henlo"), casing.apply_word("henlo", 0));
    assert_eq!(Cow::Borrowed("u"), casing.apply_word("u", 1));
    assert_eq!(Cow::Borrowed("stinky"), casing.apply_word("stinky", 2));
    assert_eq!(Cow::Borrowed("biRb"), casing.apply_word("biRb", 4));
  }

  #[test]
  fn lowercase() {
    let casing = Casing::Lowercase;
    assert_eq!(Cow::Borrowed("henlo"), casing.apply_word("henlo", 0));
    assert_eq!(Cow::Borrowed("u"), casing.apply_word("u", 1));
    assert_eq!(Cow::Borrowed("stinky"), casing.apply_word("stinky", 2));
    assert_eq!(Cow::<str>::Owned("birb".into()), casing.apply_word("biRb", 4));
  }

  #[test]
  fn uppercase() {
    let casing = Casing::Uppercase;
    assert_eq!(Cow::Borrowed("HENLO"), casing.apply_word("HENLO", 0));
    assert_eq!(Cow::<str>::Owned("U".into()), casing.apply_word("u", 1));
    assert_eq!(Cow::<str>::Owned("STINKY".into()), casing.apply_word("stinky", 2));
    assert_eq!(Cow::<str>::Owned("BIRB".into()), casing.apply_word("biRb", 4));
  }

  #[test]
  fn pascal_case() {
    let casing = Casing::PascalCase;
    assert_eq!(Cow::Borrowed("Henlo"), casing.apply_word("Henlo", 0));
    assert_eq!(Cow::<str>::Owned("U".into()), casing.apply_word("u", 1));
    assert_eq!(Cow::Borrowed("Stinky"), casing.apply_word("Stinky", 2));
    assert_eq!(Cow::<str>::Owned("Birb".into()), casing.apply_word("BiRb", 4));
  }

  #[test]
  fn camel_case() {
    let casing = Casing::CamelCase;
    assert_eq!(Cow::Borrowed("henlo"), casing.apply_word("henlo", 0));
    assert_eq!(Cow::Borrowed("henlo"), casing.apply_word("hEnlo", 0));
    assert_eq!(Cow::<str>::Owned("U".into()), casing.apply_word("u", 1));
    assert_eq!(Cow::Borrowed("Stinky"), casing.apply_word("Stinky", 2));
    assert_eq!(Cow::<str>::Owned("Birb".into()), casing.apply_word("biRb", 4));
  }
}
