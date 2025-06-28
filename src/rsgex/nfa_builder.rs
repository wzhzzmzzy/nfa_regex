use regex_syntax::hir::{Hir, HirKind, Literal};

use super::nfa::NFAutomata;

#[derive(Default)]
pub struct NFAutomataBuilder {
    pub nfa: NFAutomata,
}

impl NFAutomataBuilder {
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

    fn append(&mut self, other_nfa: NFAutomata, union_state: usize) {
        if other_nfa.states.len() < 2 {
            return;
        }

        let mut origin_nfa = self.nfa.clone();
        let origin_len = origin_nfa.states.len();

        origin_nfa.fill_state(other_nfa.states.len() - 1);
        origin_nfa.remove_ending(union_state);
        other_nfa.ending.iter().for_each(|i| {
            origin_nfa.add_ending(i + origin_len);
        });

        other_nfa
            .states
            .iter()
            .enumerate()
            .for_each(|(from, state)| {
                state.matchers.iter().for_each(|(matcher, to)| {
                    origin_nfa.add_transition(
                        if state.is_initial { union_state } else { from },
                        *to,
                        matcher.clone(),
                    )
                });
            });

        self.nfa = origin_nfa;
    }

    fn alternation(&mut self, ast_vec: &[Hir]) {
        let mut nfa = NFAutomata::new();

        nfa.fill_state(1);
        nfa.set_initial(0);

        ast_vec.iter().for_each(|ast| {
            let sub_nfa = NFAutomataBuilder::ast_to_nfa(ast.kind());
            self.append(sub_nfa.nfa, 0);
        });

        nfa.fill_state(1);

        let real_ending = nfa.states.len() - 1;
        nfa.add_ending(real_ending);

        for from in nfa.ending.clone().into_iter() {
            nfa.add_epsilon_transition(from, real_ending);
            nfa.remove_ending(from);
        }

        self.nfa = nfa;
    }

    fn concat(&mut self, ast_vec: &[Hir]) {
        let mut nfa = NFAutomata::new();

        nfa.fill_state(1);
        nfa.set_initial(0);
        nfa.add_ending(0);

        ast_vec.iter().for_each(|ast| {
            let sub_nfa = NFAutomataBuilder::ast_to_nfa(ast.kind());
            let prev_ending = nfa.ending.pop().unwrap();
            self.append(sub_nfa.nfa, prev_ending);
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

    pub fn ast_to_nfa(ast: &HirKind) -> Self {
        let mut builder = Self::default();
        match ast {
            HirKind::Alternation(ast_vec) => builder.alternation(ast_vec.as_slice()),
            HirKind::Concat(ast_vec) => builder.concat(ast_vec.as_slice()),
            HirKind::Literal(literal) => builder.literal(literal),
            _ => (),
        }

        builder
    }
}
