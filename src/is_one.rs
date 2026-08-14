/// "It's just one, isn't it?"…
pub trait IsOne {
    /// A convenience extension…
    /// 
    /// `if something.is_one() {..}` vs `if something == 1 {..}`.
    fn is_one(&self) -> bool;
}

macro_rules! implement_isone_prim {
    ([$($prim:tt),*]) => {

    };
    ($bits:tt) => {paste!{
        impl IsOne for [<i $bits>] { #[inline(always)] fn is_one(&self) -> bool {*self == 1 }}
        impl IsOne for &[<i $bits>] { #[inline(always)] fn is_one(&self) -> bool {**self == 1 }}
        impl IsOne for [<u $bits>] { #[inline(always)] fn is_one(&self) -> bool {*self == 1 }}
        impl IsOne for &[<u $bits>] { #[inline(always)] fn is_one(&self) -> bool {**self == 1 }}
    }};
}
// Implement `IsOne` for all primitive integer types.
implement_isone_prim!([8,16,32,64,128,size]);
