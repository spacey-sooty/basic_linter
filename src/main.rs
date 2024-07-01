use std::{error::Error, fs::ReadDir, path::PathBuf, str::FromStr};

use grep::{
    matcher::Matcher,
    regex::RegexMatcher,
    searcher::{sinks::UTF8, SearcherBuilder},
};

fn parse(strict: bool, dir: ReadDir) -> Result<(), Box<dyn Error>> {
    let mut searcher = SearcherBuilder::new().build();
    let nolint_matcher = RegexMatcher::new("//\\sNOLINT")?;

    let matcher = if strict {
        RegexMatcher::new("unwrap|unsafe|clone|panic|expect")? // NOLINT
    } else {
        RegexMatcher::new("unwrap|unsafe|clone")? // NOLINT
    };

    for file in dir {
        let filebuf = file?.path();

        if filebuf == PathBuf::from_str("./.git")?
            || filebuf == PathBuf::from_str("./target")?
            || filebuf == PathBuf::from_str("./Cargo.lock")?
        {
            continue;
        }

        if filebuf.metadata()?.is_dir() {
            parse(strict, std::fs::read_dir(filebuf)?)?;
            continue;
        }

        searcher.search_path(
            &matcher,
            &filebuf,
            UTF8(|num, line| {
                if !nolint_matcher.is_match(line.as_bytes())? {
                    println!(
                        "{}:{} : {}",
                        &filebuf.to_str().expect("Should be able to stringify file"),
                        num,
                        line
                    );
                    return Ok(true);
                }
                Ok(false)
            }),
        )?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let strict = match std::env::args().nth(1) {
        Some(string) => string == "strict",
        _ => false,
    };

    parse(strict, std::fs::read_dir(".")?)?;
    Ok(())
}
