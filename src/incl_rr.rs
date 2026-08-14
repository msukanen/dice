pub mod prim;

pub trait InclusiveRandomRange<T> {
    fn random_of(&self) -> T;
}
