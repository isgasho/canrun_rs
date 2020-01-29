use crate::can::pair::Pair;
use crate::{both, either, equal, with3, Goal};
use crate::{Can, CanT};

pub fn append<T: CanT>(a: Can<T>, b: Can<T>, c: Can<T>) -> Goal<T> {
    either(
        both(equal(a.clone(), Can::Nil), equal(b.clone(), c.clone())),
        with3(move |first, rest_of_a, rest_of_c| {
            both(
                both(
                    equal(a.clone(), Pair::new(first.into(), rest_of_a.into())),
                    equal(c.clone(), Pair::new(first.into(), rest_of_c.into())),
                ),
                append(rest_of_a.into(), b.clone(), rest_of_c.into()),
            )
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::append;
    use crate::can::pair::Pair;
    use crate::{Can, LVar, State};

    #[test]
    fn basic_append() {
        let state: State<Option<&str>> = State::new();
        let x = LVar::new();
        let hi = Pair::new(
            Can::Val(Some("h")),
            Pair::new(Can::Val(Some("i")), Can::Nil),
        );
        let h = Pair::new(Can::Val(Some("h")), Can::Nil);
        let i = Pair::new(Can::Val(Some("i")), Can::Nil);
        let goal = append(h, x.into(), hi);

        let mut result1 = goal.clone().run(state);
        assert_eq!(result1.nth(0).unwrap().resolve_var(x), i);
    }
}
