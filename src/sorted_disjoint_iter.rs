use std::{cmp::max, ops::RangeInclusive};

use itertools::Itertools;

use crate::{
    unsorted_disjoint::{AssumeSortedStarts, UnsortedDisjoint},
    Integer, SortedStarts,
};

#[derive(Clone)]
pub struct SortedDisjointIter<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedStarts,
{
    iter: I,
    range: Option<RangeInclusive<T>>,
}

impl<T, I> SortedDisjointIter<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedStarts,
{
    pub fn new(iter: I) -> Self {
        Self { iter, range: None }
    }
}

impl<T: Integer, const N: usize> From<[T; N]>
    for SortedDisjointIter<T, SortedRangeInclusiveVec<T>>
{
    fn from(arr: [T; N]) -> Self {
        arr.as_slice().into()
    }
}

impl<T: Integer> From<&[T]> for SortedDisjointIter<T, SortedRangeInclusiveVec<T>> {
    fn from(slice: &[T]) -> Self {
        slice.iter().cloned().collect()
    }
}

type SortedRangeInclusiveVec<T> = AssumeSortedStarts<T, std::vec::IntoIter<RangeInclusive<T>>>;

// cmk0000 these seem to specific and seem to do too much
impl<T: Integer> FromIterator<T> for SortedDisjointIter<T, SortedRangeInclusiveVec<T>> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().map(|x| x..=x).collect()
    }
}

// !!!cmk0000 name this?: SortedDisjointIter<T, AssumeSortedStarts<T, std::vec::IntoIter<RangeInclusive<T>>>>

impl<T: Integer> FromIterator<RangeInclusive<T>>
    for SortedDisjointIter<T, SortedRangeInclusiveVec<T>>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        UnsortedDisjoint::from(iter.into_iter()).into()
    }
}

impl<T, I> From<UnsortedDisjoint<T, I>> for SortedDisjointIter<T, SortedRangeInclusiveVec<T>>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>, // Any iterator is OK, because we will sort
{
    fn from(unsorted_disjoint: UnsortedDisjoint<T, I>) -> Self {
        let iter = AssumeSortedStarts {
            iter: unsorted_disjoint.sorted_by_key(|range_inclusive| *range_inclusive.start()),
        };
        Self { iter, range: None }
    }
}

impl<T: Integer, I> Iterator for SortedDisjointIter<T, I>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedStarts,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<RangeInclusive<T>> {
        if let Some(range_inclusive) = self.iter.next() {
            let (start, stop) = range_inclusive.into_inner();
            if stop < start {
                return self.next(); // !!!cmk0000 test this
            }
            if let Some(current_range_inclusive) = self.range.clone() {
                let (current_start, current_stop) = current_range_inclusive.into_inner();
                debug_assert!(current_start <= start); // cmk debug panic if not sorted
                if start <= current_stop
                    || (current_stop < T::max_value2() && start <= current_stop + T::one())
                {
                    self.range = Some(current_start..=max(current_stop, stop));
                    self.next()
                } else {
                    self.range = Some(start..=stop);
                    Some(current_start..=current_stop)
                }
            } else {
                self.range = Some(start..=stop);
                self.next()
            }
        } else {
            let result = self.range.clone();
            self.range = None;
            result
        }
    }

    // There could be a few as 1 (or 0 if the iter is empty) or as many as the iter.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.size_hint();
        let low = low.min(1);
        (low, high)
    }
}
