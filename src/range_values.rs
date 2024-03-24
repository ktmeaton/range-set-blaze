#![allow(missing_docs)]
use crate::{map::CloneBorrow, sorted_disjoint_map::RangeValue, Integer};
use alloc::{collections::btree_map, rc::Rc};
use core::{
    iter::{Enumerate, FusedIterator},
    marker::PhantomData,
    num::NonZeroUsize,
    ops::RangeInclusive,
};

use crate::{
    map::{EndValue, ValueOwned},
    sorted_disjoint_map::{SortedDisjointMap, SortedStartsMap},
};

/// An iterator that visits the ranges in the [`RangeSetBlaze`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`ranges`] method on [`RangeSetBlaze`]. See [`ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`ranges`]: crate::RangeSetBlaze::ranges
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct RangeValuesIter<'a, T: Integer, V: ValueOwned> {
    pub(crate) iter: btree_map::Iter<'a, T, EndValue<T, V>>,
}

impl<'a, T: Integer, V: ValueOwned> AsRef<RangeValuesIter<'a, T, V>> for RangeValuesIter<'a, T, V> {
    fn as_ref(&self) -> &Self {
        // Self is RangeValuesIter<'a>, the type for which we impl AsRef
        self
    }
}

// RangeValuesIter (one of the iterators from RangeSetBlaze) is SortedDisjoint
// impl<'a, T: Integer, V: ValueOwned> SortedStartsMap<'a, T, V, &'a V> for RangeValuesIter<'a, T, V> {}
// impl<'a, T: Integer, V: ValueOwned> SortedDisjointMap<'a, T, V, &'a V>
//     for RangeValuesIter<'a, T, V>
// {
// }

impl<T: Integer, V: ValueOwned> ExactSizeIterator for RangeValuesIter<'_, T, V> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T: Integer, V: ValueOwned> FusedIterator for RangeValuesIter<'a, T, V> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T, V> Iterator for RangeValuesIter<'a, T, V>
where
    T: Integer,
    V: ValueOwned + 'a,
{
    type Item = RangeValue<T, V, &'a V>; // Assuming VR is always &'a V for next

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end_value)| {
            RangeValue::new(*start..=end_value.end, &end_value.value, None)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// cmk
// impl<T: Integer, V: ValueOwned> DoubleEndedIterator for RangeValuesIter<'_, T, V, VR> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.iter.next_back().map(|(start, end)| *start..=*end)
//     }
// }

#[must_use = "iterators are lazy and do nothing unless consumed"]
/// An iterator that moves out the ranges in the [`RangeSetBlaze`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`into_ranges`] method on [`RangeSetBlaze`]. See [`into_ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`into_ranges`]: crate::RangeSetBlaze::into_ranges
pub struct IntoRangeValuesIter<T: Integer, V: ValueOwned> {
    pub(crate) iter: btree_map::IntoIter<T, EndValue<T, V>>,
}

// impl<'a, T: Integer, V: ValueOwned + 'a> SortedStartsMap<'a, T, V, Rc<V>>
//     for IntoRangeValuesIter<'a, T, V>
// {
// }
// impl<'a, T: Integer, V: ValueOwned + 'a> SortedDisjointMap<'a, T, V, Rc<V>>
//     for IntoRangeValuesIter<'a, T, V>
// {
// }

impl<'a, T: Integer, V: ValueOwned> ExactSizeIterator for IntoRangeValuesIter<T, V> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T: Integer, V: ValueOwned> FusedIterator for IntoRangeValuesIter<T, V> {}

impl<'a, T: Integer, V: ValueOwned + 'a> Iterator for IntoRangeValuesIter<T, V> {
    type Item = RangeValue<T, V, Rc<V>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end_value)| {
            let range = start..=end_value.end;
            // cmk don't use RangeValue here
            RangeValue::new(range, Rc::new(end_value.value), None)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// cmk
// impl<'a, T: Integer, V: ValueOwned> DoubleEndedIterator for IntoRangeValuesIter<'a, T, V> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.iter.next_back().map(|(start, end)| start..=end)
//     }
// }

/// cmk
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct RangeValuesToRangesIter<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    iter: I,
    option_ranges: Option<RangeInclusive<T>>,
    phantom: PhantomData<(V, VR)>,
}

// implement exact size iterator for one special case
impl<'a, T> ExactSizeIterator for RangeValuesToRangesIter<T, (), &'a (), RangeValuesIter<'a, T, ()>>
where
    T: Integer,
{
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

// // RangeValuesToRangesIter (one of the iterators from RangeSetBlaze) is SortedDisjoint
// impl<'a, T, V, VR, I> SortedStarts<T> for RangeValuesToRangesIter<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
// }
// impl<'a, T, V, VR, I> SortedDisjoint<T> for RangeValuesToRangesIter<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
// }

// impl<'a, T, V, VR, I> ops::Not for RangeValuesToRangesIter<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = NotIter<T, Self>;

//     fn not(self) -> Self::Output {
//         self.complement()
//     }
// }

impl<T, V, VR, I> FusedIterator for RangeValuesToRangesIter<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}

impl<T, V, VR, I> RangeValuesToRangesIter<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    /// Creates a new `RangeValuesToRangesIter` from an existing sorted disjoint map iterator.
    /// `option_ranges` is initialized as `None` by default.
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            option_ranges: None, // Starts as None
            phantom: PhantomData,
        }
    }
}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T, V, VR, I> Iterator for RangeValuesToRangesIter<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If no next value, return whatever is current (could be None)
            let Some(next_range_value) = self.iter.next() else {
                return self.option_ranges.take();
            };
            let (next_start, next_end) = next_range_value.range.into_inner();

            // If no current value, set current to next and loop
            let Some(current_range) = self.option_ranges.take() else {
                self.option_ranges = Some(next_start..=next_end);
                continue;
            };
            let (current_start, current_end) = current_range.into_inner();

            // If current range and next range are adjacent, merge them and loop
            if current_end + T::one() == next_start {
                self.option_ranges = Some(current_start..=next_end);
                continue;
            }

            self.option_ranges = Some(next_start..=next_end);
            return Some(current_start..=current_end);
        }
    }
}

// cmk
// impl<T: Integer, V: ValueOwned> DoubleEndedIterator for RangeValuesToRangesIter<'_, T, V> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.iter.next_back().map(|(start, end)| *start..=*end)
//     }
// }

// /// cmk
// #[derive(Clone)]
// #[must_use = "iterators are lazy and do nothing unless consumed"]
// pub struct IntoRangeValuesToRangesIter<'a, T, V, VR, I>
// where
//     T: Integer + 'a,
//     V: ValueOwned,
//     VR: CloneBorrow<V> + 'a,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     iter: I,
//     option_ranges: Option<RangeInclusive<T>>,
//     phantom0: PhantomData<&'a V>,
//     phantom1: PhantomData<VR>,
// }
// // IntoRangeValuesToRangesIter (one of the iterators from RangeSetBlaze) is SortedDisjoint
// impl<'a, T, V, VR, I> crate::SortedStarts<T> for IntoRangeValuesToRangesIter<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
// }
// impl<'a, T, V, VR, I> crate::SortedDisjoint<T> for IntoRangeValuesToRangesIter<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
// }

// impl<'a, T, V, VR, I> FusedIterator for IntoRangeValuesToRangesIter<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
// }

// impl<'a, T, V, VR, I> IntoRangeValuesToRangesIter<'a, T, V, VR, I>
// where
//     T: Integer + 'a,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     /// Creates a new `IntoRangeValuesToRangesIter` from an existing sorted disjoint map iterator.
//     /// `option_ranges` is initialized as `None` by default.
//     pub fn new(iter: I) -> Self {
//         Self {
//             iter,
//             option_ranges: None, // Starts as None
//             phantom0: PhantomData,
//             phantom1: PhantomData,
//         }
//     }
// }

// // Range's iterator is just the inside BTreeMap iterator as values
// impl<'a, T, V, VR, I> Iterator for IntoRangeValuesToRangesIter<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Item = RangeInclusive<T>;

//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             // If no next value, return whatever is current (could be None)
//             let Some(next_range_value) = self.iter.next() else {
//                 return self.option_ranges.take();
//             };
//             let (next_start, next_end) = next_range_value.range.into_inner();

//             // If no current value, set current to next and loop
//             let Some(current_range) = self.option_ranges.take() else {
//                 self.option_ranges = Some(next_start..=next_end);
//                 continue;
//             };
//             let (current_start, current_end) = current_range.into_inner();

//             // If current range and next range are adjacent, merge them and loop
//             if current_end + T::one() == next_start {
//                 self.option_ranges = Some(current_start..=next_end);
//                 continue;
//             }

//             self.option_ranges = Some(next_start..=next_end);
//             return Some(current_start..=current_end);
//         }
//     }
// }

// cmk
// impl<T: Integer, V: ValueOwned> DoubleEndedIterator for IntoRangeValuesToRangesIter<'_, T, V> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.iter.next_back().map(|(start, end)| *start..=*end)
//     }
// }

pub struct NonZeroEnumerate<I>
where
    I: Iterator,
{
    inner: Enumerate<I>,
    current_index: NonZeroUsize, // Start from 1
}

impl<I> Iterator for NonZeroEnumerate<I>
where
    I: Iterator,
{
    type Item = (NonZeroUsize, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let (_, item) = self.inner.next()?;
        let index = self.current_index;

        // Increment the current index, panic on overflow
        self.current_index = non_zero_checked_sub(self.current_index, 1)
            .expect_debug_unwrap_release("Overflow when incrementing NonZeroUsize index");

        Some((index, item))
    }
}

pub fn non_zero_checked_sub(value: NonZeroUsize, subtrahend: usize) -> Option<NonZeroUsize> {
    // Convert to usize, perform checked subtraction, then try to make a new NonZeroUsize
    value
        .get()
        .checked_sub(subtrahend)
        .and_then(NonZeroUsize::new)
}

pub(crate) trait NonZeroEnumerateExt: Iterator + Sized {
    fn non_zero_enumerate(self) -> NonZeroEnumerate<Self> {
        NonZeroEnumerate {
            inner: self.enumerate(),
            current_index: NonZeroUsize::MAX,
        }
    }
}

impl<I: Iterator> NonZeroEnumerateExt for I {}

// cmk why bother with this?
pub(crate) const NON_ZERO_MIN: NonZeroUsize = NonZeroUsize::MIN;
pub(crate) const NON_ZERO_MAX: NonZeroUsize = NonZeroUsize::MAX;

pub(crate) trait ExpectDebugUnwrapRelease<T> {
    fn expect_debug_unwrap_release(self, msg: &str) -> T;
}

#[allow(unused_variables)]
impl<T> ExpectDebugUnwrapRelease<T> for Option<T> {
    fn expect_debug_unwrap_release(self, msg: &str) -> T {
        #[cfg(debug_assertions)]
        {
            self.expect(msg)
        }
        #[cfg(not(debug_assertions))]
        {
            self.unwrap()
        }
    }
}
#[derive(Clone, Debug)]
pub struct AdjustPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = RangeValue<T, V, VR>>,
{
    iter: I,
    new_priority: Option<NonZeroUsize>,
}

impl<T, V, VR, I> Iterator for AdjustPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    type Item = RangeValue<T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|mut range_value| {
            range_value.priority = self.new_priority;
            range_value
        })
    }
}

impl<'a, T, V, VR, I> AdjustPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    pub fn new(iter: I, new_priority: Option<NonZeroUsize>) -> Self {
        AdjustPriorityMap { iter, new_priority }
    }
}

// all AdjustPriorityMap are also SortedDisjointMaps
impl<'a, T, V, VR, I> SortedStartsMap<T, V, VR> for AdjustPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}
impl<T, V, VR, I> SortedDisjointMap<T, V, VR> for AdjustPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}
