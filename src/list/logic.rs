use crate::casing::Casing;
use super::WordList;

use rand::Rng;

impl WordList {
  pub fn get(&self, n: u32) -> &str {
    self.list[&n].as_str()
  }

  pub fn roll(&self, rng: &mut impl Rng) -> u32 {
    (0..self.rolls)
      .map(|_| rng.gen_range(1, 7))
      .inspect(|n| info!("rolled a {}", n))
      .fold(0, |acc, x| 10 * acc + x)
  }

  pub fn generate(&self, rng: &mut impl Rng, words: u8, separator: &str, casing: &Casing) -> String {
    info!("generating a passphrase");

    let mut s = String::with_capacity(64);

    for i in 0..words {
      info!("generating word {}", i + 1);

      let r = self.roll(rng);
      info!("combined rolls: {}", r);

      let w = casing.apply_word(self.get(r), usize::from(i));
      info!("word: {}", w);

      s += &*w;

      if i < words - 1 {
        s += separator;
      }
    }

    s.shrink_to_fit();
    s
  }
}
