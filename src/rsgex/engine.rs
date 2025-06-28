use super::{nfa_builder::NFAutomataBuilder, parser::parse_by_regex_syntax};
use anyhow::Result;

pub struct Engine {
    nfa_builder: NFAutomataBuilder,
}

impl Engine {
    pub fn compute(&self, input: &str) -> bool {
        self.nfa_builder.nfa.compute(input)
    }

    pub fn new(pattern: &str) -> Result<Self> {
        let ast = parse_by_regex_syntax(pattern)?;

        let nfa = NFAutomataBuilder::ast_to_nfa(ast.kind());

        Ok(Engine { nfa_builder: nfa })
    }
}

// fn regex_to_nfa(ast: Hir) {
//     match ast.kind() {
//         HirKind::Alternation(alternatives) => {},
//         HirKind::Concat
//         _ => (),
//     }
// }
