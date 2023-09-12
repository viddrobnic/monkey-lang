use crate::object::Object;

pub trait Evaluate {
    fn evaluate(&self) -> Object;
}
