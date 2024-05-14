use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{Integer, SortedDisjoint};

/// Turns a [`SortedDisjoint`] iterator into a [`SortedDisjoint`] iterator of its complement,
/// i.e., all the integers not in the original iterator, as sorted & disjoint ranges.
///
/// # Example
///
/// ```
/// use range_set_blaze::{prelude::*, NotIter};
///
/// let a = CheckSortedDisjoint::new([1u8..=2, 5..=100]);
/// let b = NotIter::new(a);
/// assert_eq!(b.into_string(), "0..=0, 3..=4, 101..=255");
///
/// // Or, equivalently:
/// let b = !CheckSortedDisjoint::new([1u8..=2, 5..=100]);
/// assert_eq!(b.into_string(), "0..=0, 3..=4, 101..=255");
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    iter: I,
    start_not: T,
    next_time_return_none: bool,
}

impl<T, I> NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    /// Create a new [`NotIter`] from a [`SortedDisjoint`] iterator. See [`NotIter`] for an example.
    pub fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = RangeInclusive<T>, IntoIter = I>,
    {
        NotIter {
            iter: iter.into_iter(),
            start_not: T::min_value2(),
            next_time_return_none: false,
        }
    }
}

impl<T, I> FusedIterator for NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T> + FusedIterator,
{
}

impl<T, I> Iterator for NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    type Item = RangeInclusive<T>;
    fn next(&mut self) -> Option<RangeInclusive<T>> {
        debug_assert!(T::min_value2() <= T::max_value2()); // real assert
        if self.next_time_return_none {
            return None;
        }
        let next_item = self.iter.next();
        if let Some(range) = next_item {
            let (start, end) = range.into_inner();
            debug_assert!(start <= end);
            if self.start_not < start {
                // We can subtract with underflow worry because
                // we know that start > start_not and so not min_value
                let result = Some(self.start_not..=start.sub_one());
                if end < T::max_value2() {
                    self.start_not = end.add_one();
                } else {
                    self.next_time_return_none = true;
                }
                result
            } else if end < T::max_value2() {
                self.start_not = end.add_one();
                self.next() // will recurse at most once
            } else {
                self.next_time_return_none = true;
                None
            }
        } else {
            self.next_time_return_none = true;
            Some(self.start_not..=T::max_value2())
        }
    }

    // We could have one less or one more than the iter.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.size_hint();
        let low = if low > 0 { low - 1 } else { 0 };
        let high = high.map(|high| {
            if high < usize::MAX {
                high + 1
            } else {
                usize::MAX
            }
        });
        (low, high)
    }
}

// FUTURE define Not, etc on DynSortedDisjoint
