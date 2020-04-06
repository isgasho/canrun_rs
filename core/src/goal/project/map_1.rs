use super::Goal;
use super::Project;
use crate::domains::{Domain, DomainType};
use crate::state::State;
use crate::state::Watch;
use crate::unify::Unify;
use crate::value::{
    IntoVal, Val,
    Val::{Resolved, Var},
};
use std::fmt;
use std::rc::Rc;

/// Create a [projection goal](super) that allows deriving one resolved value
/// from the other.
///
/// Functions must be provided to derive in both directions. Whichever value is
/// resolved first will be used to derive the other.
///
/// ```
/// use canrun::{Goal, all, unify, var, map_1};
/// use canrun::domains::example::I32;
///
/// let (x, y) = (var(), var());
/// let goal: Goal<I32> = all![
///     unify(1, x),
///     map_1(x, y, |x| x + 1, |y| y - 1),
/// ];
/// let result: Vec<_> = goal.query(y).collect();
/// assert_eq!(result, vec![2])
/// ```
pub fn map_1<'a, A: 'a, AV, B: 'a, BV, D, AtoB, BtoA>(
    a: AV,
    b: BV,
    a_to_b: AtoB,
    b_to_a: BtoA,
) -> Goal<'a, D>
where
    AV: IntoVal<A>,
    BV: IntoVal<B>,
    D: Domain<'a> + DomainType<'a, A> + DomainType<'a, B>,
    State<'a, D>: Unify<'a, A> + Unify<'a, B>,
    AtoB: Fn(&A) -> B + 'a,
    BtoA: Fn(&B) -> A + 'a,
{
    Goal::Project(Rc::new(Map1 {
        a: a.into_val(),
        b: b.into_val(),
        a_to_b: Rc::new(a_to_b),
        b_to_a: Rc::new(b_to_a),
    }))
}

pub struct Map1<'a, A, B> {
    a: Val<A>,
    b: Val<B>,
    a_to_b: Rc<dyn Fn(&A) -> B + 'a>,
    b_to_a: Rc<dyn Fn(&B) -> A + 'a>,
}

impl<'a, A, B> fmt::Debug for Map1<'a, A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Map1 ??")
    }
}

impl<'a, A, B, Dom> Project<'a, Dom> for Map1<'a, A, B>
where
    Dom: Domain<'a> + DomainType<'a, A> + DomainType<'a, B> + 'a,
    State<'a, Dom>: Unify<'a, A> + Unify<'a, B>,
{
    fn attempt<'r>(&'r self, state: State<'a, Dom>) -> Watch<State<'a, Dom>> {
        let a = state.resolve_val(&self.a).clone();
        let b = state.resolve_val(&self.b).clone();
        match (a, b) {
            (Resolved(a), b) => {
                let f = &self.a_to_b;
                Watch::done(state.unify(f(&*a).into_val(), b))
            }
            (a, Resolved(b)) => {
                let f = &self.b_to_a;
                Watch::done(state.unify(f(&*b).into_val(), a))
            }
            (Var(a), Var(b)) => Watch::watch(state, a).and(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::map_1;
    use crate::domains::example::I32;
    use crate::goal::unify::unify;
    use crate::goal::Goal;
    use crate::util;
    use crate::value::var;

    #[test]
    fn succeeds() {
        let x = var();
        let y = var();
        let goals: Vec<Goal<I32>> =
            vec![unify(1, x), unify(2, y), map_1(x, y, |x| x + 1, |y| y - 1)];
        util::all_permutations_resolve_to(goals, (x, y), vec![(1, 2)]);
    }
}
