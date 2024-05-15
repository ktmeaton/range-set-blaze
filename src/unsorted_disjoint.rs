use crate::{Integer, RangeSetBlaze, SortedDisjoint, SortedStarts};
use core::{
    cmp::{max, min},
    iter::FusedIterator,
    ops::RangeInclusive,
};
use num_traits::Zero;

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    iter: I,
    option_range: Option<RangeInclusive<T>>,
    min_value_plus_2: T,
}

impl<T, I> UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>, // Any iterator is fine
{
    pub fn new(iter: I) -> Self {
        UnsortedDisjoint {
            iter,
            option_range: None,
            min_value_plus_2: T::min_value().add_one().add_one(),
        }
    }
}

impl<T, I> FusedIterator for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
}

impl<T, I> Iterator for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let range = match self.iter.next() {
                Some(r) => r,
                None => return self.option_range.take(),
            };

            let (next_start, next_end) = range.into_inner();
            if next_start > next_end {
                continue;
            }
            let Some(self_range) = self.option_range.clone() else {
                self.option_range = Some(next_start..=next_end);
                continue;
            };

            let (self_start, self_end) = self_range.into_inner();
            if (next_start >= self.min_value_plus_2 && self_end <= next_start.sub_one().sub_one())
                || (self_start >= self.min_value_plus_2
                    && next_end <= self_start.sub_one().sub_one())
            {
                let result = Some(self_start..=self_end);
                self.option_range = Some(next_start..=next_end);
                return result;
            }
            self.option_range = Some(min(self_start, next_start)..=max(self_end, next_end));
            // continue;
        }
    }

    // As few as one (or zero if iter is empty) and as many as iter.len()
    // There could be one extra if option_range is Some.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let lower = if lower == 0 { 0 } else { 1 };
        if self.option_range.is_some() {
            (lower, upper.map(|x| x + 1))
        } else {
            (lower, upper)
        }
    }
}

// cmk00 does every iterator have this?
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct SortedDisjointWithLenSoFar<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    iter: I,
    len: <T as Integer>::SafeLen,
}

impl<T, I> SortedDisjointWithLenSoFar<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint<T>,
{
    pub fn new(iter: I) -> Self {
        SortedDisjointWithLenSoFar {
            iter,
            len: T::SafeLen::zero(),
        }
    }
}

impl<T: Integer, I> SortedDisjointWithLenSoFar<T, I>
where
    I: SortedDisjoint<T>,
{
    pub fn len_so_far(&self) -> <T as Integer>::SafeLen {
        self.len
    }
}

impl<T: Integer, I> FusedIterator for SortedDisjointWithLenSoFar<T, I> where
    I: SortedDisjoint<T> + FusedIterator
{
}

impl<T: Integer, I> Iterator for SortedDisjointWithLenSoFar<T, I>
where
    I: SortedDisjoint<T>,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range) = self.iter.next() {
            let (start, end) = range.clone().into_inner();
            debug_assert!(start <= end);
            self.len += T::safe_len(&range);
            Some((start, end))
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]

/// Gives any iterator of ranges the [`SortedStarts`] trait without any checking.
pub struct AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    pub(crate) iter: I,
}

impl<T, I> FusedIterator for AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
}

impl<T: Integer, I> SortedStarts<T> for AssumeSortedStarts<T, I> where
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator
{
}

impl<T, I> AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    /// Construct [`AssumeSortedStarts`] from a range iterator.
    pub fn new<J: IntoIterator<IntoIter = I>>(iter: J) -> Self {
        AssumeSortedStarts {
            iter: iter.into_iter(),
        }
    }

    /// Create a [`RangeSetBlaze`] from an [`AssumeSortedStarts`] iterator.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = RangeSetBlaze::from_sorted_starts(AssumeSortedStarts::new([-10..=-5, -7..=2]));
    /// let a1: RangeSetBlaze<i32> = AssumeSortedStarts::new([-10..=-5, -7..=2]).into_range_set_blaze();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=2");
    /// ```
    pub fn into_range_set_blaze(self) -> RangeSetBlaze<T>
    where
        Self: Sized,
    {
        RangeSetBlaze::from_sorted_starts(self)
    }
}

impl<T, I> Iterator for AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
