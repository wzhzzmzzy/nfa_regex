use super::{nfa::NFAutomata, parser};
use anyhow::Result;
use regex_syntax::hir::{Class, Hir, HirKind, Literal, Repetition};

#[derive(Default)]
pub struct Engine {
    pub nfa: NFAutomata,
}

impl Engine {
    // FIXME: fn one_step should be private
    pub fn one_step(&mut self, char_or_epsilon: Option<char>) {
        let mut nfa = NFAutomata::new();

        nfa.declare_state(2, 0, 1);

        if let Some(c) = char_or_epsilon {
            nfa.add_char_transition(0, 1, c);
        } else {
            nfa.add_epsilon_transition(0, 1);
        }

        self.nfa = nfa;
    }

    fn alternation(&mut self, ast_vec: &[Hir]) {
        let mut nfa = NFAutomata::new();

        nfa.fill_state(1);
        nfa.set_initial(0);

        ast_vec.iter().for_each(|ast| {
            let sub_nfa = Self::ast_to_nfa(ast.kind());
            nfa.append(&sub_nfa.nfa, 0);
        });

        nfa.fill_state(1);

        let real_ending = nfa.states.len() - 1;
        for from in nfa.ending.clone().into_iter() {
            nfa.add_epsilon_transition(from, real_ending);
            nfa.remove_ending(from);
        }
        nfa.add_ending(real_ending);

        self.nfa = nfa;
    }

    fn concat(&mut self, ast_vec: &[Hir]) {
        let mut nfa = NFAutomata::new();

        nfa.fill_state(1);
        nfa.set_initial(0);
        nfa.add_ending(0);

        ast_vec.iter().for_each(|ast| {
            let sub_nfa = Self::ast_to_nfa(ast.kind());
            let prev_ending = nfa.ending.pop().unwrap();
            nfa.append(&sub_nfa.nfa, prev_ending);
        });

        self.nfa = nfa;
    }

    fn literal(&mut self, literal: &Literal) {
        let mut nfa = NFAutomata::new();

        let len = literal.0.len();
        nfa.declare_state(len + 1, 0, len);
        literal
            .0
            .iter()
            .map(|&b| b as char)
            .enumerate()
            .for_each(|(from, c)| {
                nfa.add_char_transition(from, from + 1, c);
            });

        self.nfa = nfa;
    }

    fn repetition(&mut self, repetition: &Repetition) {
        let mut nfa = NFAutomata::new();
        nfa.fill_state(1);
        nfa.set_initial(0);
        nfa.add_ending(0);

        let sub_nfa = Self::ast_to_nfa(repetition.sub.kind());

        let mut last_sub_nfa_initial: usize = 0;
        for _ in 0..repetition.min {
            last_sub_nfa_initial = nfa.ending.pop().unwrap();
            nfa.append(&sub_nfa.nfa, last_sub_nfa_initial);
        }

        if let Some(max) = repetition.max {
            let mut sub_nfa_ending: Vec<usize> = vec![];
            for _ in repetition.min..max {
                let current_sub_nfa_ending = nfa.ending.pop().unwrap();
                nfa.append(&sub_nfa.nfa, current_sub_nfa_ending);
                sub_nfa_ending.push(current_sub_nfa_ending);
            }
            for ending in sub_nfa_ending.into_iter() {
                nfa.add_epsilon_transition(ending, *nfa.ending.last().unwrap());
            }
        } else {
            nfa.add_epsilon_transition(*nfa.ending.last().unwrap(), last_sub_nfa_initial);
        }

        self.nfa = nfa;
    }

    /// TODO: Class contains multipul unicode/byte ranges, character matcher will be too weak to
    /// represent ranges, maybe should support range matcher for Class
    fn class(&mut self, _class: &Class) {
        todo!()
    }

    pub fn ast_to_nfa(ast: &HirKind) -> Self {
        let mut builder = Self::default();
        // TODO:
        // Look: ^ / $
        // Capture: ()
        // Repetition: greedy +? / *? ...
        // Class: . / [] / [^]
        match ast {
            HirKind::Alternation(ast_vec) => builder.alternation(ast_vec.as_slice()),
            HirKind::Concat(ast_vec) => builder.concat(ast_vec.as_slice()),
            HirKind::Literal(literal) => builder.literal(literal),
            HirKind::Repetition(repetition) => builder.repetition(repetition),
            HirKind::Class(class) => builder.class(class),
            _ => (),
        }

        println!("ast_to_nfa, {:?}", ast);
        builder.nfa.debug();

        builder
    }
}

impl TryFrom<&str> for Engine {
    type Error = anyhow::Error;

    fn try_from(pattern: &str) -> Result<Engine, Self::Error> {
        let ast = parser::parse_by_regex_syntax(pattern);
        let e = Engine::ast_to_nfa(ast?.kind());

        Ok(e)
    }
}

#[cfg(test)]
mod test {
    use super::Engine;

    #[test]
    fn test_literal() {
        let e = Engine::try_from("123").unwrap();
        assert!(e.nfa.compute("123"));
        assert!(!e.nfa.compute("124"));
    }

    #[test]
    fn test_alternation() {
        let e = Engine::try_from("123|456").unwrap();
        assert!(e.nfa.compute("123"));
        assert!(e.nfa.compute("456"));
        assert!(!e.nfa.compute("345"));
    }

    #[test]
    fn test_repetition() {
        let e = Engine::try_from("1+").unwrap();

        assert!(e.nfa.compute("1"));
        assert!(!e.nfa.compute("1"));
        assert!(e.nfa.compute("11"));
        assert!(e.nfa.compute("111"));
    }

    #[test]
    fn test_repetition_complex() {
        let e = Engine::try_from("1+2+3+4{2}").unwrap();

        assert!(e.nfa.compute("11122233344"));
        assert!(e.nfa.compute("111222333445"));
        assert!(!e.nfa.compute("22244"));
        assert!(!e.nfa.compute("11122233345"));

        let e2 = Engine::try_from("1234{1,5}").unwrap();

        assert!(e2.nfa.compute("123444455"));
    }

    #[test]
    fn test_accepter() {
        let e = Engine::try_from("123+|456+|7{3}|888").unwrap();

        assert!(e.nfa.compute("888"));
    }
}
