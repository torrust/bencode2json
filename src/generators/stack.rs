//! The stack used by the generators to keep track of the current parsing state.
use std::fmt::Display;

/// Stack containing states for nested Bencoded values.
///
/// The stack has an immutable initial state.
///
/// > NOTICE!: It's not allowed to pop or change the initial state.
#[derive(Debug)]
pub(crate) struct Stack {
    /// The stack of states.
    states: Vec<State>,
}

/// States while parsing list or dictionaries.
///
/// There are no states for integers and strings because it's a straightforward
/// operation. We know when they finish and there is no recursion.
///
/// States are displayed with a short name using only one letter:
///
/// `I`, `L`, `M`, `D`, `E`, `F`
///
/// This comes from the original implementation in C.
#[derive(Debug, PartialEq, Clone)]
pub enum State {
    /// The initial state.
    /// /// The sort display name for the state is L.
    Initial, // I

    // States while parsing lists
    /// Expecting the first list item or the end of the list.
    /// The sort display name for the state is L.
    ExpectingFirstListItemOrEnd,

    /// Expecting the next list item. List contains at least one item.
    /// The sort display name for the state is M.
    ExpectingNextListItem,

    // States while parsing dictionaries
    /// Expecting the first dict field or the end of the dict.
    /// The sort display name for the state is D.
    ExpectingFirstDictFieldOrEnd,

    /// Expecting the dict field value.
    /// The sort display name for the state is E.
    ExpectingDictFieldValue,

    /// Expecting the dict field key or the end of the dict.
    /// The sort display name for the state is F.
    ExpectingDictFieldKeyOrEnd,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            State::Initial => "I",
            State::ExpectingFirstListItemOrEnd => "L",
            State::ExpectingNextListItem => "M",
            State::ExpectingFirstDictFieldOrEnd => "D",
            State::ExpectingDictFieldValue => "E",
            State::ExpectingDictFieldKeyOrEnd => "F",
        };
        write!(f, "{output}")
    }
}

impl Default for Stack {
    fn default() -> Self {
        let states = vec![State::Initial];
        Self { states }
    }
}

impl Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (idx, state) in <std::vec::Vec<State> as Clone>::clone(&self.states)
            .into_iter()
            .enumerate()
        {
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{state}")?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl Stack {
    /// It adds a new state to the stack.
    pub fn push(&mut self, state: State) {
        self.states.push(state);
    }

    /// It returns and consumes the stack top.
    ///
    /// It doesn't allow popping the initial state.
    ///
    /// # Panics
    ///
    /// Will panic is the stack state is the initial state.
    pub fn pop(&mut self) {
        self.guard_immutable_initial_state();
        self.states.pop();
    }

    /// It swaps the stack top with the new state.
    ///
    /// It doesn't allow swapping the initial state.
    ///
    /// # Panics
    ///
    /// Will panic is the stack state is the initial state.
    pub fn swap_top(&mut self, new_state: State) {
        self.guard_immutable_initial_state();
        self.states.pop();
        self.push(new_state);
    }

    /// It returns the top element on the stack without consuming it.
    ///
    /// # Panics
    ///
    /// Will panic is the stack is empty. The stack is never empty because it's
    /// not allowed to pop or change the initial state.
    #[must_use]
    pub fn peek(&self) -> State {
        match self.states.last() {
            Some(top) => top.clone(),
            None => panic!("empty stack!"),
        }
    }

    /// Prevent from mutating the initial state.
    fn guard_immutable_initial_state(&self) {
        if let Some(top) = self.states.last() {
            if *top != State::Initial {
                return;
            }
        };

        panic!("trying to mutate immutable initial state. It can't be popped or swapped!")
    }
}

#[cfg(test)]
mod tests {
    mod the_stack_state {
        use crate::generators::stack::State;

        #[test]
        fn should_be_displayed_with_single_letter_abbreviations() {
            assert_eq!(format!("{}", State::Initial), "I");
            assert_eq!(format!("{}", State::ExpectingFirstListItemOrEnd), "L");
            assert_eq!(format!("{}", State::ExpectingNextListItem), "M");
            assert_eq!(format!("{}", State::ExpectingFirstDictFieldOrEnd), "D");
            assert_eq!(format!("{}", State::ExpectingDictFieldValue), "E");
            assert_eq!(format!("{}", State::ExpectingDictFieldKeyOrEnd), "F");
        }
    }

    mod the_stack {
        mod it_should {
            use crate::generators::stack::{Stack, State};

            #[test]
            fn have_an_initial_state() {
                assert_eq!(Stack::default().peek(), State::Initial);
            }

            #[test]
            fn allow_peeking_the_top_element_without_consuming_it() {
                let stack = Stack::default();

                let _ = stack.peek();

                assert_eq!(stack.peek(), State::Initial);
            }

            #[test]
            #[should_panic(expected = "empty stack!")]
            fn panic_peeking_the_top_element_if_the_stack_is_empty() {
                let mut stack = Stack::default();

                stack.states.clear();
                let _ = stack.peek();

                assert_eq!(stack.peek(), State::Initial);
            }

            #[test]
            fn allow_pushing_new_states() {
                let mut stack = Stack::default();

                stack.push(State::ExpectingDictFieldKeyOrEnd);

                assert_eq!(stack.peek(), State::ExpectingDictFieldKeyOrEnd);
            }

            #[test]
            fn allow_popping_the_current_top_state() {
                let mut stack = Stack::default();

                stack.push(State::ExpectingDictFieldKeyOrEnd);
                stack.pop();

                assert_eq!(stack.peek(), State::Initial);
            }

            #[test]
            #[should_panic(expected = "trying to mutate")]
            fn not_allow_popping_the_initial_state() {
                Stack::default().pop();
            }

            #[test]
            fn allow_swapping_the_top_state() {
                let mut stack = Stack::default();

                stack.push(State::ExpectingDictFieldKeyOrEnd);
                stack.swap_top(State::ExpectingDictFieldValue);

                assert_eq!(stack.peek(), State::ExpectingDictFieldValue);
            }

            #[test]
            #[should_panic(expected = "trying to mutate")]
            fn not_allow_swapping_the_initial_state() {
                Stack::default().swap_top(State::Initial);
            }

            mod be_displayed_with_single_letter_abbreviations_for_states {

                use crate::generators::stack::{Stack, State};

                #[test]
                fn with_the_initial_state() {
                    let stack = Stack::default();

                    assert_eq!(format!("{stack}"), "[I]");
                }

                #[test]
                fn after_pushing_one_more_state() {
                    let mut stack = Stack::default();

                    stack.push(State::ExpectingDictFieldKeyOrEnd);

                    assert_eq!(format!("{stack}"), "[I, F]");
                }
            }
        }
    }
}
