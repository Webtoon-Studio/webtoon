pub(crate) trait MathExt {
    /// Returns the 1-based index of the bucket that `self` falls into, given a bucket size.
    ///
    /// This method is useful for determining which "bucket" (e.g., page, group, segment)
    /// an item belongs to when items are grouped in fixed-size chunks. A value of `0`
    /// is treated as belonging to the first bucket.
    ///
    /// If `size` is `0`, the method returns `1` by default (to avoid division by zero),
    /// but you may want to guard against this explicitly depending on your use case.
    ///
    /// Using the largest available number for a bucket size will return the total number of buckets
    /// there are e.g. if there are 652 episodes and 9 episodes per page, that would result in
    /// 73 pages, the total amount of pages.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(0u32.in_bucket_of(20), 1);      // value 0 belongs to bucket 1
    /// assert_eq!(7u32.in_bucket_of(20), 1);      // value 7 belongs to bucket 1
    /// assert_eq!(19u32.in_bucket_of(20), 1);     // value 19 belongs to bucket 1
    /// assert_eq!(20u32.in_bucket_of(20), 1);     // value 20 belongs to bucket 1
    /// assert_eq!(21u32.in_bucket_of(20), 2);     // value 21 belongs to bucket 2
    /// assert_eq!(331u32.in_bucket_of(20), 17);   // value 331 belongs to bucket 17
    /// assert_eq!(653u32.in_bucket_of(20), 33);   // value 653 belongs to bucket 33
    /// ```
    fn in_bucket_of(self, size: Self) -> Self;
}

macro_rules! impl_math_ext {
    ($($T:ty),*) => {
        $(
            #[allow(clippy::cast_lossless)]
            impl MathExt for $T {
                #[inline(always)]
                fn in_bucket_of(self, size: Self) -> Self {
                    if size == 0 {
                        return 1;
                    }
                    self / size + (self % size != 0 || self == 0) as $T
                }
            }
        )*
    };
}

impl_math_ext!(u8, u16, u32, u64, usize, i32, i64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_calculate_proper_bucket() {
        assert_eq!(1, 0.in_bucket_of(20));
        assert_eq!(1, 7.in_bucket_of(20));
        assert_eq!(1, 19.in_bucket_of(20));
        assert_eq!(1, 20.in_bucket_of(20));
        assert_eq!(2, 21.in_bucket_of(20));
        assert_eq!(1, 15.in_bucket_of(15));
        assert_eq!(73, 652.in_bucket_of(9));
        assert_eq!(9, 89.in_bucket_of(10));
    }
}
