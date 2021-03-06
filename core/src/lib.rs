#![warn(missing_docs)]
//! Canrun is a [logic
//! programming](https://en.wikipedia.org/wiki/Logic_programming) library
//! inspired by the [\*Kanren](http://minikanren.org/) family of language DSLs.
//!
//! ## Status: Exploratory and Highly Experimental
//!
//! I'm still quite new to both Rust and logic programming, so there are likely
//! to be rough edges. At best the goal is to be a useful implementation of the
//! core concepts of a Kanren in way that is idiomatic to Rust. At worst it may
//! just be a poor misinterpretation with fatal flaws.
//!
//! ## Quick Start
//!
//! ```rust
//! use canrun::{Goal, both, unify, var};
//! use canrun::domains::example::I32;
//!
//! let x = var();
//! let y = var();
//! let goal: Goal<I32> = both(unify(x, y), unify(1, x));
//! let result: Vec<_> = goal.query(y).collect();
//! assert_eq!(result, vec![1])
//! ```

pub mod domains;
pub mod goal;
mod impls;
mod query;
pub mod state;
mod unify;
pub mod value;

#[doc(inline)]
pub use domains::{Domain, DomainType};
#[doc(inline)]
pub use goal::project::{assert_1, assert_2, map_1, map_2, project_1, project_2};
#[doc(inline)]
pub use goal::{both, custom, either, lazy, unify, Goal};
#[doc(inline)]
pub use query::Query;
#[doc(inline)]
pub use state::{Fork, IterResolved, ResolvedState, State, StateIter};
#[doc(inline)]
pub use unify::UnifyIn;
#[doc(inline)]
pub use value::{var, IntoVal, LVar, ReifyIn, Val};

pub use impls::tuples::*;

#[doc(inline)]
pub use domains::domain;

pub mod util;

#[cfg(test)]
mod tests {
    mod test_constrain;
    mod test_fork;
    mod test_unify;
}

// #[macro_use]
// extern crate log;
