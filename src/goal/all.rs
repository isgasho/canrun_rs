use super::Goal;
use crate::domain::Domain;
use crate::state::State;

pub(crate) fn run<'a, D>(state: State<'a, D>, goals: Vec<Goal<'a, D>>) -> Option<State<'a, D>>
where
    D: Domain<'a>,
{
    goals.into_iter().try_fold(state, |s, g| g.apply(s))
}

pub fn all<'a, D>(goals: Vec<Goal<'a, D>>) -> Goal<'a, D>
where
    D: Domain<'a>,
{
    Goal::All(goals)
}

#[cfg(test)]
mod tests {
    use super::all;
    use crate::goal::unify::unify;
    use crate::tests::util;
    use crate::value::{val, var};

    #[test]
    fn succeeds() {
        let x = var();
        let y = var();
        let goal = all(vec![unify(x.clone(), val(5)), unify(y.clone(), val(7))]);
        let result = util::goal_resolves_to(goal, (&x, &y));
        assert_eq!(result, vec![(5, 7)]);
    }

    #[test]
    fn fails() {
        let x = var();
        let goal = all(vec![unify(x, 5), unify(x, 7)]);
        let result = util::goal_resolves_to(goal.clone(), &x);
        assert_eq!(result, vec![]);
    }
}
