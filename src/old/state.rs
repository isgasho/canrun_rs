use crate::can::lvar::LVar;
use crate::can::{pair, vec, Can, CanT};
use crate::constraint::Constraint;
use crate::goal::StateIter;
use crate::util::multikeymultivaluemap::MKMVMap as MultiMap;

use im::{HashMap, HashSet};
use std::iter::{empty, once};

#[derive(Clone, Debug)]
pub struct State<'a, T: CanT> {
    pub(crate) values: HashMap<LVar, Can<T>>,
    pub(crate) constraints: MultiMap<LVar, Constraint<'a, T>>,
}

impl<'a, T: CanT + 'a> State<'a, T> {
    pub fn new() -> State<'a, T> {
        State {
            values: HashMap::new(),
            constraints: MultiMap::new(),
        }
    }

    pub(crate) fn to_iter(self) -> StateIter<'a, T> {
        Box::new(once(self))
    }

    // pub(crate) fn clone_iter(&self) -> StateIter<'a, T> {
    //     Box::new(once(self.clone()))
    // }

    pub(crate) fn assign(&self, var: LVar, value: Can<T>) -> Self {
        State {
            values: self.values.update(var, value),
            constraints: self.constraints.clone(),
        }
    }

    pub(crate) fn checked_resolve(
        &self,
        can: &Can<T>,
        history: &HashSet<LVar>,
    ) -> ResolveResult<T> {
        match can {
            Can::Var(lvar) => {
                if history.contains(lvar) {
                    debug!("{:?}", history);
                    Err(UnifyError::InfiniteRecursion(*lvar))
                } else {
                    let history = history.update(*lvar);
                    match self.values.get(lvar) {
                        Some(val) => self.checked_resolve(val, &history),
                        None => Ok(can.clone()),
                    }
                }
            }
            Can::Val(v) => Ok(Can::Val(v.clone())),
            Can::Pair { l, r } => pair::resolve(self, l, r, history),
            Can::Vec(v) => vec::resolve(self, v, history),
            Can::Nil => Ok(Can::Nil),
        }
    }

    pub fn resolve(&self, can: &Can<T>) -> ResolveResult<T> {
        self.checked_resolve(can, &HashSet::new())
    }

    pub fn resolve_var(&self, key: LVar) -> ResolveResult<T> {
        self.resolve(&Can::Var(key))
    }

    pub fn unify(self, a: Can<T>, b: Can<T>) -> StateIter<'a, T> {
        self.try_unify(a, b).unwrap_or_else(|err| {
            debug!("{:?}", err);
            empty_iter()
        })
    }

    fn try_unify(self, a_: Can<T>, b_: Can<T>) -> UnifyResult<'a, T> {
        let a = self.resolve(&a_)?;
        let b = self.resolve(&b_)?;

        Ok(if a == b {
            Box::new(once(self.clone())) as StateIter<T>
        } else {
            match (a, b) {
                (Can::Var(av), bv) => Box::new(self.assign(av, bv).check_constraints(av)),
                (av, Can::Var(bv)) => Box::new(self.assign(bv, av).check_constraints(bv)),
                (Can::Pair { l: al, r: ar }, Can::Pair { l: bl, r: br }) => {
                    pair::unify(self, *al, *ar, *bl, *br)
                }
                (Can::Vec(a), Can::Vec(b)) => vec::unify(self, a, b),
                _ => empty_iter(),
            }
        })
    }
}

pub(crate) fn empty_iter<'a, T: CanT + 'a>() -> StateIter<'a, T> {
    Box::new(empty())
}

#[derive(Debug)]
pub enum UnifyError {
    InfiniteRecursion(LVar),
}

pub type ResolveResult<T> = Result<Can<T>, UnifyError>;
pub type UnifyResult<'a, T> = Result<StateIter<'a, T>, UnifyError>;

#[cfg(test)]
mod tests {
    use crate::{var, Can, LVar, State};
    use im::HashMap;

    #[test]
    fn new() {
        let state: State<u8> = State::new();
        assert_eq!(state.values, HashMap::new());
    }

    #[test]
    fn assign() {
        let state: State<u8> = State::new();
        let x = LVar::new();
        let state = state.assign(x, Can::Val(5));
        assert_eq!(state.values, HashMap::unit(x, Can::Val(5)));
    }

    #[test]
    fn value_of_direct() {
        let state: State<u8> = State::new();
        let x = LVar::new();
        let state = state.assign(x, Can::Val(5));
        assert_eq!(state.resolve_var(x).unwrap(), Can::Val(5));
    }

    #[test]
    fn value_of_missing() {
        let state: State<u8> = State::new();
        let x = var();
        assert_eq!(state.resolve_var(x).unwrap(), x.can());
        assert_eq!(state.resolve(&Can::Val(5)).unwrap(), Can::Val(5));
    }
    #[test]
    fn value_of_nested() {
        let state: State<u8> = State::new();
        let x = LVar::new();
        let y = LVar::new();
        let z = LVar::new();
        let state = state.assign(x, Can::Var(y));
        let state = state.assign(y, Can::Var(z));
        let state = state.assign(z, Can::Val(5));

        assert_eq!(state.resolve_var(x).unwrap(), Can::Val(5));
        assert_eq!(state.resolve_var(y).unwrap(), Can::Val(5));
        assert_eq!(state.resolve_var(z).unwrap(), Can::Val(5));
    }
    #[test]
    fn unify_with_self() {
        let state: State<u8> = State::new();
        let x = var();
        let unified = state.clone().unify(x.can(), x.can()).nth(0);
        assert_eq!(unified.unwrap().values, state.values);
    }
    #[test]
    fn unify_two_vars() {
        let state: State<u8> = State::new();
        let x = LVar::new();
        let y = LVar::new();

        assert_eq!(
            state
                .clone()
                .unify(Can::Var(x), Can::Var(y))
                .nth(0)
                .unwrap()
                .values,
            state.assign(x, Can::Var(y)).values
        );
    }
    #[test]
    fn unify_with_value() {
        let x = LVar::new();
        let state: State<u8> = State::new();

        assert_eq!(
            (state.clone())
                .unify(Can::Var(x), Can::Val(5))
                .nth(0)
                .unwrap()
                .values,
            state.assign(x, Can::Val(5)).values
        );
        assert_eq!(
            (state.clone())
                .unify(Can::Val(5), Can::Var(x))
                .nth(0)
                .unwrap()
                .values,
            state.assign(x, Can::Val(5)).values
        );
    }
    #[test]
    fn unify_already_bound() {
        let x = LVar::new();
        let state: State<u8> = State::new().assign(x, Can::Val(5));
        let result = state.unify(Can::Var(x), Can::Val(6)).nth(0);
        assert!(result.is_none());
    }
}