mod lvar;

pub(super) use lvar::LVarId;
pub use lvar::{var, LVar};
use std::fmt;
use std::rc::Rc;

pub enum Val<T: ?Sized> {
    Var(LVar<T>),
    Resolved(Rc<T>),
}

use Val::{Resolved, Var};

impl<T> Val<T> {
    pub fn resolved(&self) -> Result<&T, LVar<T>> {
        match self {
            Resolved(x) => Ok(&*x),
            Var(x) => Err(*x),
        }
    }
}

pub trait IntoVal<T> {
    fn into_val(self) -> Val<T>;
}

impl<T> IntoVal<T> for T {
    fn into_val(self) -> Val<T> {
        Val::Resolved(Rc::new(self))
    }
}

impl<T> IntoVal<T> for Val<T> {
    fn into_val(self) -> Val<T> {
        self
    }
}

impl<T> IntoVal<T> for &Val<T> {
    fn into_val(self) -> Val<T> {
        self.clone()
    }
}

impl<T: Clone> IntoVal<T> for &T {
    fn into_val(self) -> Val<T> {
        Val::Resolved(Rc::new(self.clone()))
    }
}

impl<T> IntoVal<T> for LVar<T> {
    fn into_val(self) -> Val<T> {
        Val::Var(self)
    }
}
impl<T> IntoVal<T> for &LVar<T> {
    fn into_val(self) -> Val<T> {
        Val::Var(self.clone())
    }
}
impl<T> LVar<T> {
    pub fn into_val(&self) -> Val<T> {
        Val::Var(self.clone())
    }
}

#[macro_export]
macro_rules! val {
    ($value:expr) => {
        canrun::value::IntoVal::into_val($value)
    };
}

#[doc(inline)]
pub use val;

impl<T> Clone for Val<T> {
    fn clone(&self) -> Self {
        match self {
            Val::Var(var) => Val::Var(*var),
            Val::Resolved(r) => Val::Resolved(r.clone()),
        }
    }
}

impl<T: PartialEq> PartialEq for Val<T> {
    fn eq(&self, other: &Val<T>) -> bool {
        match (self, other) {
            (Resolved(s), Resolved(other)) => s == other,
            (Var(s), Var(other)) => s == other,
            _ => false,
        }
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for Val<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resolved(v) => write!(f, "Resolved({:?})", v),
            Var(v) => write!(f, "Var({:?})", v),
        }
    }
}
