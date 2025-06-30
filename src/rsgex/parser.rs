use anyhow::Result;
use regex_syntax::{hir::Hir, parse};

pub fn parse_by_regex_syntax(pattern: &str) -> Result<Hir> {
    let ast = parse(pattern)?;

    Ok(ast)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_by_regex_syntax() {
        println!("{:?}", parse_by_regex_syntax("@(?<region>.+).com"));
    }
}
