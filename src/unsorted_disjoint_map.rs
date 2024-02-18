use crate::{
    map::EndValue,
    sorted_disjoint_map::{RangeValue, SortedDisjointMap, SortedStartsMap},
    Integer,
};
use alloc::collections::btree_map::Range;
use core::{
    cmp::{max, min},
    iter::FusedIterator,
    marker::PhantomData,
    ops::RangeInclusive,
};
use num_traits::Zero;

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct UnsortedDisjointMap<'a, T, V, I>
where
    T: Integer,
    V: PartialEq + 'a,
    I: Iterator<Item = RangeValue<T, &'a V>>,
{
    iter: I,
    option_range: Option<RangeValue<T, &'a V>>,
    min_value_plus_2: T,
    two: T,
}

impl<'a, T, V, I> From<I> for UnsortedDisjointMap<'a, T, V, I::IntoIter>
where
    T: Integer,
    V: PartialEq + 'a,
    I: IntoIterator<Item = RangeValue<T, &'a V>>, // Any iterator is fine
{
    fn from(into_iter: I) -> Self {
        UnsortedDisjointMap {
            iter: into_iter.into_iter(),
            option_range: None,
            min_value_plus_2: T::min_value() + T::one() + T::one(),
            two: T::one() + T::one(),
        }
    }
}

// cmk
// impl<'a, T, V, I> FusedIterator for UnsortedDisjointMap<'a, T, V, I>
// where
//     T: Integer,
//     V: PartialEq + 'a,
//     I: Iterator<Item = RangeValue<T, &'a V>> + FusedIterator,
// {
// }

impl<'a, T, V, I> Iterator for UnsortedDisjointMap<'a, T, V, I>
where
    T: Integer,
    V: PartialEq + 'a,
    I: Iterator<Item = RangeValue<T, &'a V>>,
{
    type Item = RangeValue<T, &'a V>;

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
            assert!(
                next_end <= T::safe_max_value(),
                "end must be <= T::safe_max_value()"
            );

            let Some(self_range) = self.option_range.clone() else {
                self.option_range = Some(next_start..=next_end);
                continue;
            };

            let (self_start, self_end) = self_range.into_inner();
            if (next_start >= self.min_value_plus_2 && self_end <= next_start - self.two)
                || (self_start >= self.min_value_plus_2 && next_end <= self_start - self.two)
            {
                let result = Some(self_start..=self_end);
                self.option_range = Some(next_start..=next_end);
                return result;
            } else {
                self.option_range = Some(min(self_start, next_start)..=max(self_end, next_end));
                continue;
            }
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

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct SortedDisjointWithLenSoFarMap<'a, T, V, I>
where
    T: Integer,
    V: PartialEq + Clone + 'a,
    I: SortedDisjointMap<'a, T, V>,
{
    iter: I,
    len: <T as Integer>::SafeLen,
    _phantom_data: PhantomData<&'a V>,
}

// cmk
// impl<T: Integer, V: PartialEq, I> From<I> for SortedDisjointWithLenSoFarMap<T, V, I::IntoIterMap>
// where
//     I: IntoIterator<Item = RangeValue<T, V>>,
//     I::IntoIter: SortedDisjointMap<T, V>,
// {
//     fn from(into_iter: I) -> Self {
//         SortedDisjointWithLenSoFarMap {
//             iter: into_iter.into_iter(),
//             len: <T as Integer>::SafeLen::zero(),
//             _phantom_data: PhantomData,
//         }
//     }
// }

impl<'a, T: Integer, V: PartialEq + Clone + 'a, I> SortedDisjointWithLenSoFarMap<'a, T, V, I>
where
    I: SortedDisjointMap<'a, T, V>,
{
    pub fn len_so_far(&self) -> <T as Integer>::SafeLen {
        self.len
    }
}

// cmk
// impl<T: Integer, V: PartialEq, I> FusedIterator for SortedDisjointWithLenSoFarMap<T, V, I> where
//     I: SortedDisjointMap<T, V> + FusedIterator
// {
// }

impl<'a, T: Integer, V: PartialEq + Clone + 'a, I> Iterator
    for SortedDisjointWithLenSoFarMap<'a, T, V, I>
where
    I: SortedDisjointMap<'a, T, V>,
{
    type Item = (T, EndValue<T, V>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range_value) = self.iter.next() {
            let (start, end) = range_value.range.clone().into_inner();
            debug_assert!(start <= end && end <= T::safe_max_value());
            self.len += T::safe_len(&range_value.range);
            let end_value = EndValue {
                end,
                value: range_value.value.clone(),
            };
            Some((start, end_value))
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

/// Gives any iterator of ranges the [`SortedStartsMap`] trait without any checking.
#[doc(hidden)]
pub struct AssumeSortedStartsMap<'a, T, V, I>
where
    T: Integer,
    V: PartialEq + 'a,
    I: Iterator<Item = RangeValue<T, &'a V>>,
{
    pub(crate) iter: I,
}

impl<'a, T: Integer, V: PartialEq + 'a, I> SortedStartsMap<'a, T, V>
    for AssumeSortedStartsMap<'a, T, V, I>
where
    I: Iterator<Item = RangeValue<T, &'a V>>,
{
}

impl<'a, T, V, I> AssumeSortedStartsMap<'a, T, V, I>
where
    T: Integer,
    V: PartialEq + 'a,
    I: Iterator<Item = RangeValue<T, &'a V>>,
{
    pub fn new(iter: I) -> Self {
        AssumeSortedStartsMap { iter }
    }
}

impl<'a, T, V, I> FusedIterator for AssumeSortedStartsMap<'a, T, V, I>
where
    T: Integer,
    V: PartialEq + 'a,
    I: Iterator<Item = RangeValue<T, &'a V>> + FusedIterator,
{
}

impl<'a, T, V, I> Iterator for AssumeSortedStartsMap<'a, T, V, I>
where
    T: Integer,
    V: PartialEq + 'a,
    I: Iterator<Item = RangeValue<T, &'a V>>,
{
    type Item = RangeValue<T, &'a V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T: Integer, V: PartialEq + Clone + 'a, I> From<I>
    for SortedDisjointWithLenSoFarMap<'a, T, V, I::IntoIter>
where
    I: IntoIterator<Item = RangeValue<T, &'a V>>,
    I::IntoIter: SortedDisjointMap<'a, T, V>,
{
    fn from(into_iter: I) -> Self {
        SortedDisjointWithLenSoFarMap {
            iter: into_iter.into_iter(),
            len: <T as Integer>::SafeLen::zero(),
            _phantom_data: PhantomData,
        }
    }
}
