use crate::state;
use crate::util::multikeyvaluemap::Value as MultiMapValue;
use crate::{all, Can, CanT, Equals, Goal, LVar, State, StateIter};

#[derive(Clone, PartialEq, Debug)]
pub struct Mapping<T: CanT> {
    pub a: Can<T>,
    pub b: Can<T>,
    pub a_to_b: fn(T) -> T,
    pub b_to_a: fn(T) -> T,
}

pub(crate) trait DirtyImmutable<T> {
    fn clone_and_push(&self, t: T) -> Self;
}
impl<T: Clone> DirtyImmutable<T> for Vec<T> {
    fn clone_and_push(&self, t: T) -> Self {
        let mut cloned = self.to_vec();
        cloned.push(t);
        cloned
    }
}

impl<'a, T: CanT + 'a> Mapping<T> {
    pub fn run(self, state: State<T>) -> StateIter<'a, T> {
        match (self.a.clone(), self.b.clone()) {
            (Can::Var(a), Can::Var(b)) => {
                Box::new(state.add_mappings(vec![a, b], self).check_mappings(a.can()))
            }
            (Can::Var(a), _) => Box::new(state.add_mappings(vec![a], self).check_mappings(a.can())),
            (_, Can::Var(b)) => Box::new(state.add_mappings(vec![b], self).check_mappings(b.can())),
            // TODO: either side is a val?
            (Can::Val(a), Can::Val(b)) => {
                if self.evaluate_a(a) == b {
                    state.to_iter()
                } else {
                    state::empty_iter()
                }
            }
            _ => state::empty_iter(),
        }
    }

    pub fn evaluate_a(&self, a: T) -> T {
        let func = self.a_to_b;
        func(a)
    }

    pub fn evaluate_b(&self, b: T) -> T {
        let func = self.b_to_a;
        func(b)
    }
}

impl<'a, T: CanT + 'a> State<T> {
    pub(crate) fn add_mappings(&self, vars: Vec<LVar>, mappings: Mapping<T>) -> Self {
        State {
            values: self.values.clone(),
            constraints: self.constraints.clone(),
            mappings: self.mappings.set(vars, mappings),
        }
    }

    pub(crate) fn add_mappings_key(
        &self,
        key: LVar,
        mappings: &MultiMapValue<LVar, Mapping<T>>,
    ) -> Self {
        State {
            values: self.values.clone(),
            constraints: self.constraints.clone(),
            mappings: self.mappings.add_key(key, mappings),
        }
    }

    pub(crate) fn remove_mapping(&self, mappings: &MultiMapValue<LVar, Mapping<T>>) -> Self {
        State {
            values: self.values.clone(),
            constraints: self.constraints.clone(),
            mappings: self.mappings.remove(mappings),
        }
    }

    pub(crate) fn check_mappings(self, can: Can<T>) -> StateIter<'a, T> {
        match can {
            Can::Var(lvar) => {
                let mappings = self.mappings.get(&lvar);
                let satisfied =
                    mappings
                        .iter()
                        .try_fold((self.clone(), vec![]), |(state, goals), found| {
                            let mappings = &found.value;
                            match (
                                self.resolve(&mappings.a).ok()?,
                                self.resolve(&mappings.b).ok()?,
                            ) {
                                (Can::Val(a), Can::Val(b)) => {
                                    if mappings.evaluate_a(a) == b {
                                        Some((state.remove_mapping(found), goals))
                                    } else {
                                        None
                                    }
                                }
                                (Can::Val(val), Can::Var(var)) => {
                                    let mapped = mappings.evaluate_a(val);
                                    Some((
                                        state.remove_mapping(found),
                                        goals.clone_and_push(var.equals(Can::Val(mapped))),
                                    ))
                                }
                                (Can::Var(var), Can::Val(val)) => {
                                    let mapped = mappings.evaluate_b(val);
                                    Some((
                                        state.remove_mapping(found),
                                        goals.clone_and_push(var.equals(Can::Val(mapped))),
                                    ))
                                }
                                (Can::Var(a), _) => {
                                    if a == lvar {
                                        Some((state, goals))
                                    } else {
                                        Some((state.add_mappings_key(a, found), goals))
                                    }
                                }
                                (_, Can::Var(b)) => {
                                    if b == lvar {
                                        Some((state, goals))
                                    } else {
                                        Some((state.add_mappings_key(b, found), goals))
                                    }
                                }
                                _ => None,
                            }
                        });
                match satisfied {
                    Some((state, goals)) => all(goals).run(state),
                    None => state::empty_iter(),
                }
            }
            // Base is not an LVar. This depends on the correct base LVar being
            // maintained in the mappings store.
            _ => self.to_iter(),
        }
    }
}

pub fn map<'a, T: CanT>(
    a: Can<T>,
    b: Can<T>,
    a_to_b: fn(T) -> T,
    b_to_a: fn(T) -> T,
) -> Goal<'a, T> {
    Goal::Map(Mapping {
        a,
        b,
        a_to_b,
        b_to_a,
    })
}

#[cfg(test)]
mod tests {
    use super::map;
    use crate::util::test;
    use crate::{var, Can, Equals, Goal};

    fn increment<'a>(a: Can<usize>, b: Can<usize>) -> Goal<'a, usize> {
        map(a, b, |a| a + 1, |b| b - 1)
    }

    #[test]
    fn should_succeed_all_defined() {
        let (x, y) = (var(), var());
        let goals = vec![increment(x.can(), y.can()), x.equals(1), y.equals(2)];
        let expected = vec![vec![Can::Val(1), Can::Val(2)]];
        test::all_permutations_resolve_to(goals, &vec![x, y], expected);
    }

    #[test]
    fn should_succeed_forward() {
        let (x, y) = (var(), var());
        let goals = vec![increment(x.can(), y.can()), x.equals(1)];
        let expected = vec![vec![Can::Val(1), Can::Val(2)]];
        test::all_permutations_resolve_to(goals, &vec![x, y], expected);
    }

    #[test]
    fn should_succeed_backward() {
        let (x, y) = (var(), var());
        let goals = vec![increment(x.can(), y.can()), y.equals(2)];
        let expected = vec![vec![Can::Val(1), Can::Val(2)]];
        test::all_permutations_resolve_to(goals, &vec![x, y], expected);
    }

    #[test]
    fn should_succeed_forward_multiple_steps() {
        let (x, y, x2, y2) = (var(), var(), var(), var());
        let goals = vec![
            increment(x.can(), y2.can()),
            x.equals(x2.can()),
            x2.equals(1),
            y.equals(y2.can()),
        ];
        let expected = vec![vec![Can::Val(1), Can::Val(2)]];
        test::all_permutations_resolve_to(goals, &vec![x, y], expected);
    }

    #[test]
    fn should_succeed_backward_multiple_steps() {
        let (x, y, x2, y2) = (var(), var(), var(), var());
        let goals = vec![
            increment(x.can(), y2.can()),
            x.equals(x2.can()),
            y2.equals(2),
            y.equals(y2.can()),
        ];
        let expected = vec![vec![Can::Val(1), Can::Val(2)]];
        test::all_permutations_resolve_to(goals, &vec![x, y], expected);
    }

    #[test]
    fn should_fail() {
        let (x, y) = (var(), var());
        let goals = vec![increment(x.can(), y.can()), x.equals(1), y.equals(3)];
        let expected = vec![];
        test::all_permutations_resolve_to(goals, &vec![x, y], expected);
    }
    #[test]
    fn should_fail_multiple_steps() {
        let (x, y, x2, y2) = (var(), var(), var(), var());
        let goals = vec![
            increment(x.can(), y2.can()),
            x.equals(x2.can()),
            y2.equals(3),
            x2.equals(1),
            y.equals(y2.can()),
        ];
        let expected = vec![];
        test::all_permutations_resolve_to(goals, &vec![x, y], expected);
    }
}