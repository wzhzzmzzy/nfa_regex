use super::parser::parse_by_regex_syntax;
use anyhow::Result;
use regex_syntax::hir::{Hir, HirKind};

pub struct Engine {}

impl Engine {
    pub fn new(pattern: &str) -> Result<Self> {
        let ast = parse_by_regex_syntax(pattern)?;

        Ok(Engine {})
    }
}

// fn regex_to_nfa(ast: Hir) {
//     match ast.kind() {
//         HirKind::Alternation(alternatives) => {},
//         HirKind::Concat
//         _ => (),
//     }
// }
