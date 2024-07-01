use std::{error::Error, fs::ReadDir};

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

        if filebuf.metadata()?.is_dir() {
            parse(strict, std::fs::read_dir(filebuf)?)?;
            continue;
        }

        match filebuf.extension() {
            Some(ext) => {
                if ext != "rs" {
                    continue;
                }
            }
            None => continue,
        }

        searcher.search_path(
            &matcher,
            &filebuf,
            UTF8(|num, line| {
                if !nolint_matcher.is_match(line.as_bytes())? {
                    print!(
                        "{}:{} : {}",
                        &filebuf.to_str().expect("Should be able to stringify file"),
                        num,
                        line
                    );
                }
                Ok(true)
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
