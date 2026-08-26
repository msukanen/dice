pub mod prim_irr;
pub mod prim_rr;

pub trait InclusiveRandomRange<T> {
    fn random_of(&self) -> T;
}

pub trait RandomRange<T> {
    fn random_of(&self) -> T;
}
