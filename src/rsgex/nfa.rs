use std::{collections::VecDeque, rc::Rc};

#[derive(Clone)]
pub struct NFAutomata {
    pub states: Vec<State>,
    pub initial: usize,
    pub ending: Vec<usize>,
}

impl NFAutomata {
    pub fn new() -> Self {
        Self {
            states: vec![],
            initial: 0,
            ending: vec![],
        }
    }

    pub fn compute(&self, input: &str) -> bool {
        let mut stack: Vec<(usize, usize, Vec<usize>)> = vec![(0, self.initial, vec![])];
        let input_chars: Vec<char> = input.chars().collect();

        while let Some((i, current_state_name, epsilon_mem)) = stack.pop() {
            let current_state = self.states.get(current_state_name).unwrap();

            if current_state.is_ending {
                return true;
            }

            current_state
                .matchers
                .iter()
                .filter(|(m, _)| {
                    if i < input_chars.len() {
                        m.matches(input_chars[i])
                    } else {
                        m.is_epsilon()
                    }
                })
                .rev()
                .for_each(|(matcher, to_state_name)| {
                    if matcher.is_epsilon() {
                        if epsilon_mem.iter().all(|name| *name != *to_state_name) {
                            let mut mem = epsilon_mem.clone();
                            mem.push(*to_state_name);
                            stack.push((i, *to_state_name, mem));
                        }
                    } else {
                        stack.push((i + 1, *to_state_name, vec![]));
                    }
                });
        }

        false
    }

    pub fn set_initial(&mut self, initial: usize) {
        let state_value = self.states.get_mut(initial);

        if let Some(state) = state_value {
            self.initial = initial;
            state.is_initial = true;
        }
    }

    pub fn add_ending(&mut self, ending: usize) {
        let state_value = self.states.get_mut(ending);

        if let Some(state) = state_value {
            state.is_ending = true;
            self.ending.push(ending);
        }
    }

    pub fn remove_ending(&mut self, old: usize) {
        let old_state_value = self.states.get_mut(old);
        if let Some(state) = old_state_value {
            state.is_ending = false;
            self.ending.retain(|&x| x != old);
        }
    }

    pub fn add_transition(
        &mut self,
        from_state: usize,
        to_state: usize,
        transition: Rc<dyn Matcher>,
    ) {
        let state_value = self.states.get_mut(from_state);

        if let Some(from) = state_value {
            from.matchers.push_back((transition, to_state));
        }
    }

    pub fn unshift_transition(
        &mut self,
        from_state: usize,
        to_state: usize,
        transition: Rc<dyn Matcher>,
    ) {
        let state_value = self.states.get_mut(from_state);

        if let Some(from) = state_value {
            from.matchers.push_front((transition, to_state));
        }
    }

    pub fn fill_state(&mut self, number: usize) {
        for _ in 0..number {
            self.add_state();
        }
    }

    pub fn declare_state(&mut self, number: usize, initial: usize, ending: usize) {
        self.fill_state(number);
        self.set_initial(initial);
        self.add_ending(ending);
    }

    fn add_state(&mut self) {
        self.states.push(create_state());
    }

    pub fn add_char_transition(&mut self, from: usize, to: usize, c: char) {
        self.add_transition(from, to, Rc::new(CharacterMatcher { c }))
    }

    pub fn add_epsilon_transition(&mut self, from: usize, to: usize) {
        self.add_transition(from, to, Rc::new(EpsilonMatcher {}))
    }

    pub fn append(&mut self, other_nfa: &NFAutomata, union_state: usize) {
        if other_nfa.states.len() < 2 {
            return;
        }

        let origin_len = self.states.len();

        self.fill_state(other_nfa.states.len() - 1);
        self.remove_ending(union_state);
        other_nfa.ending.iter().for_each(|i| {
            self.add_ending(i + origin_len - 1);
        });

        other_nfa
            .states
            .iter()
            .enumerate()
            .for_each(|(from, state)| {
                state.matchers.iter().for_each(|(matcher, to)| {
                    self.add_transition(
                        if state.is_initial {
                            union_state
                        } else {
                            from + origin_len - 1
                        },
                        (*to) + origin_len - 1,
                        matcher.clone(),
                    )
                });
            });
    }

    pub fn debug(&self) {
        println!("NFA Debug Information:");
        println!("======================");

        for (index, state) in self.states.iter().enumerate() {
            let mut state_info = format!("State({})", index);

            // 添加标记
            let mut markers = Vec::new();
            if state.is_initial {
                markers.push("INITIAL");
            }
            if state.is_ending {
                markers.push("ENDING");
            }

            if !markers.is_empty() {
                state_info.push_str(&format!(" [{}]", markers.join(", ")));
            }

            // 添加转换关系
            if state.matchers.is_empty() {
                println!("{}: (no transitions)", state_info);
            } else {
                let transitions: Vec<String> = state
                    .matchers
                    .iter()
                    .map(|(matcher, to_state)| format!("--{}-> {}", matcher.label(), to_state))
                    .collect();

                println!("{}: {}", state_info, transitions.join(" "));
            }
        }

        println!("======================");
    }
}

impl Default for NFAutomata {
    fn default() -> Self {
        Self::new()
    }
}

fn create_state() -> State {
    State {
        matchers: VecDeque::new(),
        is_initial: false,
        is_ending: false,
    }
}

#[derive(Clone)]
pub struct State {
    pub matchers: VecDeque<(Rc<dyn Matcher>, usize)>,
    pub is_initial: bool,
    pub is_ending: bool,
}

pub trait Matcher {
    fn matches(&self, c: char) -> bool;
    fn is_epsilon(&self) -> bool;
    fn label(&self) -> String;
}

pub struct CharacterMatcher {
    c: char,
}
impl Matcher for CharacterMatcher {
    fn matches(&self, c: char) -> bool {
        self.c == c
    }

    fn is_epsilon(&self) -> bool {
        false
    }

    fn label(&self) -> String {
        self.c.to_string()
    }
}
pub struct EpsilonMatcher {}
impl Matcher for EpsilonMatcher {
    fn matches(&self, _c: char) -> bool {
        true
    }

    fn is_epsilon(&self) -> bool {
        true
    }

    fn label(&self) -> String {
        "ε".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfa_with_example() {
        let mut nfa = NFAutomata::default();

        nfa.declare_state(4, 0, 3);
        nfa.add_char_transition(0, 1, 'a');
        nfa.add_char_transition(1, 2, 'b');
        nfa.add_char_transition(2, 2, 'b');
        nfa.add_epsilon_transition(2, 3);

        assert!(nfa.compute("abbbbbb"));
        assert!(!nfa.compute("aabbbbbb"));
        assert!(nfa.compute("ab"));
        assert!(!nfa.compute("a"));
    }

    #[test]
    fn test_nfa_epsilon_loop() {
        let mut nfa = NFAutomata::default();

        nfa.declare_state(3, 0, 2);
        nfa.add_char_transition(0, 1, 'a');
        nfa.add_epsilon_transition(1, 1);
        nfa.add_char_transition(1, 2, 'b');

        assert!(nfa.compute("ab"));
    }
}
