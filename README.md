# passphrase

*Generate passphrases using [EFF's word lists][eff].*

Pretty much everything is customisable, but the default output looks like this:
`countable.reference.glorified.varnish.porthole.scoreless`.

What else is there to say?

    passphrase 1.1.0
    Kyle Clemens <passphrase@kyleclemens.com>
    Generates passphrases using dice rolls on word lists a la https://www.eff.org/dice.

    USAGE:
        passphrase [FLAGS] [OPTIONS]

    FLAGS:
        -h, --help       print help information
        -L, --lists      print available lists and their information then exit
        -v, --version    print version information
            --verbose    turn on verbose mode

    OPTIONS:
        -l, --list <list>              which list to pick words from [default: long]  [possible values: long, short1,
                                    short2, diceware, beale]
        -n, --amount <passphrases>     how many passphrases to generate [default: 1]
        -s, --separator <separator>    which character (or characters) to separate the words in the passphrases [default: .]
        -w, --words <words>            how many words to use in the passphrase [default: 6]

    Please see https://www.eff.org/dice to learn about the word lists embedded in this program.

[eff]: https://www.eff.org/dice
