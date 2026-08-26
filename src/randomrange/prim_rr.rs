use crate::{InclusiveRandomRange, RandomRange};

impl RandomRange<i32> for std::ops::Range<i32> {
    /// Generate random value within the given range.
    ///
    /// ```
    /// use dicebag::RandomRange;
    /// let range = 6..12;
    /// let roll = range.random_of();
    /// ```
    fn random_of(&self) -> i32 {
        let (mut start, mut end) = (self.start, self.end);
        if start > end {
            std::mem::swap(&mut start, &mut end);
            log::warn!("Might consider swapping your start/end around - they're in descending order. Did that for you here, but I'll also will keep nagging about that until you sort it out, mkay?");
        } else if start == end {
            panic!("Empty range! Not going to handwave anything arbitrary, ergo 'dicebag' will check out here until you fix the call site…");
        }
        (start..=end-1).random_of()
    }
}
