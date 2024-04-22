use core::{iter::FusedIterator, ops::RangeInclusive};

use itertools::{Itertools, KMergeBy, MergeBy};

use crate::{Integer, SortedDisjoint, SortedStarts};

/// Works with [`UnionIter`] to turn any number of [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
///
/// Also see [`KMerge`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`UnionIter`]: crate::UnionIter
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{UnionIter, Merge, SortedDisjointMap, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = UnionIter::new2(a, b);
/// assert_eq!(union.into_string(), "1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let c = a | b;
/// assert_eq!(c.into_string(), "1..=100")
/// ```
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    #[allow(clippy::type_complexity)]
    iter: MergeBy<L, R, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool>,
}

impl<T, L, R> Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    /// Creates a new [`Merge`] iterator from two [`SortedDisjointMap`] iterators. See [`Merge`] for more details and examples.
    pub fn new(left: L, right: R) -> Self {
        Self {
            iter: left.merge_by(right, |a, b| a.start() < b.start()),
        }
    }
}

impl<T, L, R> FusedIterator for Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
}

impl<T, L, R> Iterator for Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, L, R> SortedStarts<T> for Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
}

/// Works with [`UnionIter`] to turn two [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
///
/// Also see [`Merge`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`UnionIter`]: crate::UnionIter
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{UnionIter, KMerge, MultiwaySortedDisjoint, SortedDisjointMap, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::new([2..=6].into_iter());
/// let c = CheckSortedDisjoint::new([-1..=-1].into_iter());
/// let union = UnionIter::new_k([a, b, c]);
/// assert_eq!(union.into_string(), "-1..=-1, 1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::new([2..=6].into_iter());
/// let c = CheckSortedDisjoint::new([-1..=-1].into_iter());
/// let union = [a, b, c].union();
/// assert_eq!(union.into_string(), "-1..=-1, 1..=100");
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct KMerge<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    #[allow(clippy::type_complexity)]
    iter: KMergeBy<I, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool>,
}

impl<T, I> KMerge<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    /// Creates a new [`KMerge`] iterator from zero or more [`SortedDisjointMap`] iterators. See [`KMerge`] for more details and examples.
    pub fn new<K>(iter: K) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        let iter = iter.into_iter();
        // Merge RangeValues by start with ties broken by priority
        let iter: KMergeBy<I, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool> =
            iter.kmerge_by(|a, b| a.start() < b.start());
        Self { iter }
    }
}

impl<T, I> FusedIterator for KMerge<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
}

impl<T, I> Iterator for KMerge<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, I> SortedStarts<T> for KMerge<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
}
