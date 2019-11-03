 - ignore
     + Based in [this](https://github.com/rust-lang-nursery/glob/issues/59) discussion  seems that there is not good walk-with-glob-suport alternative in rust more than ignore. In the future we may use glob as it seems that they will pick it up. That will reduce the dependency footprint too.
 - zip
     + Is the main library. Seems more or less maintained. Still is a candidate for replacement in case it doesn't gets more attention.
 - clap
     + Argument parsing lib. Properly maintained.
     + We activated the YAML option in order to provide traductions in the future.
     + Maybe in the future we will migrate to roff-rs and have clap + man pages. More info [here](https://github.com/clap-rs/clap/issues/552)
