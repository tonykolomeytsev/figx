use super::{CodeBlockBuilder, PropertySpecBuilder};

pub trait Touch
where
    Self: Sized,
{
    fn touch(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }
}

impl Touch for CodeBlockBuilder {}
impl Touch for PropertySpecBuilder {}
