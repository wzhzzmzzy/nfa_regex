use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

#[derive(Clone)]
pub struct NFAutomata {
    pub states: Vec<State>,
    pub initial: usize,
    pub ending: Vec<usize>,
}

// left: usize, right: Option<usize>, group_name: Rc<str>
#[derive(Debug)]
pub struct CaptureGroupRange(usize, Option<usize>, Option<Rc<str>>);

// char_index: usize, current_state_index: usize, epsilon_mem: Vec<usize>, group_flag: u64
struct StackFrame(usize, usize, Vec<usize>, u64);

impl NFAutomata {
    pub fn new() -> Self {
        Self {
            states: vec![],
            initial: 0,
            ending: vec![],
        }
    }

    pub fn compute(&self, input: &str) -> Option<HashMap<usize, String>> {
        let mut stack: Vec<StackFrame> = vec![StackFrame(0, self.initial, vec![], 0)];
        let input_chars: Vec<char> = input.chars().collect();
        let mut group_map: HashMap<usize, CaptureGroupRange> = HashMap::new();
        let mut captured_group_flag: u64 = 0;

        while let Some(StackFrame(i, current_state_index, epsilon_mem, mut group_flag)) =
            stack.pop()
        {
            let current_state = self.states.get(current_state_index).unwrap();

            current_state
                .start_group
                .iter()
                .for_each(|(group_index, name)| {
                    group_flag |= 1 << group_index;
                    let key = *group_index as usize;
                    group_map
                        .entry(key)
                        .or_insert(CaptureGroupRange(i, None, name.clone()));
                });

            current_state.end_group.iter().for_each(|(group_index, _)| {
                if group_flag & 1 << group_index != 0 {
                    let key = *group_index as usize;
                    let capture_result = group_map.get_mut(&key).unwrap();
                    capture_result.1 = Some(i);

                    group_flag &= !(1 << group_index);
                    captured_group_flag &= 1 << group_index;
                }
            });

            if current_state.is_ending {
                // 创建一个新的HashMap来存储捕获组的字符串结果
                let mut group_captured: HashMap<usize, String> = HashMap::new();

                // 遍历所有捕获组，提取对应的字符串
                for (group_index, CaptureGroupRange(left, right_opt, _)) in &group_map {
                    if let Some(right) = right_opt {
                        // 只处理有完整范围的捕获组
                        let captured_text: String =
                            input.chars().skip(*left).take(right - left).collect();
                        group_captured.insert(*group_index, captured_text);
                    }
                }

                return Some(group_captured);
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
                            stack.push(StackFrame(i, *to_state_name, mem, group_flag));
                        }
                    } else {
                        stack.push(StackFrame(i + 1, *to_state_name, vec![], group_flag));
                    }
                });
        }

        None
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
                let from_state = if state.is_initial {
                    union_state
                } else {
                    from + origin_len - 1
                };

                if !state.start_group.is_empty() {
                    state.start_group.iter().for_each(|group| {
                        self.mark_start_capture_group(from_state, group.0, group.1.clone());
                    });
                }

                if !state.end_group.is_empty() {
                    state.end_group.iter().for_each(|group| {
                        self.mark_end_capture_group(from_state, group.0, group.1.clone());
                    });
                }

                state.matchers.iter().for_each(|(matcher, to)| {
                    let to_state = (*to) + origin_len - 1;

                    self.add_transition(from_state, to_state, matcher.clone())
                });
            });
    }

    pub fn mark_start_capture_group(
        &mut self,
        state_index: usize,
        capture_index: u32,
        name: Option<Rc<str>>,
    ) {
        if let Some(state) = self.states.get_mut(state_index) {
            state.start_group.push((capture_index, name.clone()));
        }
    }

    pub fn mark_end_capture_group(
        &mut self,
        state_index: usize,
        capture_index: u32,
        name: Option<Rc<str>>,
    ) {
        if let Some(state) = self.states.get_mut(state_index) {
            state.end_group.push((capture_index, name.clone()));
        }
    }

    pub fn mark_capture_group(&mut self, index: u32, name: Option<Rc<str>>) {
        self.mark_start_capture_group(self.initial, index, name.clone());

        self.ending.clone().into_iter().for_each(|i| {
            self.mark_end_capture_group(i, index, name.clone());
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

            // 添加capture group信息
            let mut capture_info = Vec::new();

            if !state.start_group.is_empty() {
                let start_groups: Vec<String> = state
                    .start_group
                    .iter()
                    .map(|(index, name)| {
                        if let Some(name) = name {
                            format!("START({}:{})", index, name)
                        } else {
                            format!("START({})", index)
                        }
                    })
                    .collect();
                capture_info.extend(start_groups);
            }

            if !state.end_group.is_empty() {
                let end_groups: Vec<String> = state
                    .end_group
                    .iter()
                    .map(|(index, name)| {
                        if let Some(name) = name {
                            format!("END({}:{})", index, name)
                        } else {
                            format!("END({})", index)
                        }
                    })
                    .collect();
                capture_info.extend(end_groups);
            }

            if !capture_info.is_empty() {
                state_info.push_str(&format!(" {{{}}}", capture_info.join(", ")));
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
        start_group: Vec::new(),
        end_group: Vec::new(),
    }
}

#[derive(Clone)]
pub struct State {
    pub matchers: VecDeque<(Rc<dyn Matcher>, usize)>,
    pub is_initial: bool,
    pub is_ending: bool,
    pub start_group: Vec<(u32, Option<Rc<str>>)>,
    pub end_group: Vec<(u32, Option<Rc<str>>)>,
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

        assert!(nfa.compute("abbbbbb").is_some());
        assert!(nfa.compute("aabbbbbb").is_none());
        assert!(nfa.compute("ab").is_some());
        assert!(nfa.compute("a").is_none());
    }

    #[test]
    fn test_nfa_epsilon_loop() {
        let mut nfa = NFAutomata::default();

        nfa.declare_state(3, 0, 2);
        nfa.add_char_transition(0, 1, 'a');
        nfa.add_epsilon_transition(1, 1);
        nfa.add_char_transition(1, 2, 'b');

        assert!(nfa.compute("ab").is_some());
    }
}
