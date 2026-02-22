pub mod database;
pub mod errors;
pub mod users;

pub trait ExternMethod
where
    Self: Sized,
{
    fn apply<F>(self, method: F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        method(self)
    }
    fn apply_with<F, O>(self, method: F, options: O) -> Self
    where
        F: Fn(Self, O) -> Self,
    {
        method(self, options)
    }
}

impl<T> ExternMethod for T {}
