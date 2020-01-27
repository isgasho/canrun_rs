use super::Goal;
use crate::Can;

pub fn equal<T: Eq + Clone>(a: Can<T>, b: Can<T>) -> Goal<T> {
    Goal::Equal { a, b }
}

#[cfg(test)]
mod tests {
    use super::equal;
    use crate::{Can, LVar, State};
    #[test]
    fn basic_equal() {
        let state: State<u32> = State::new();
        let x = LVar::new();
        let goal = equal(Can::Var(x), Can::Val(5));
        let mut result = goal.run(state);
        assert_eq!(result.nth(0).unwrap().resolve_var(x), Can::Val(5));
    }
}
