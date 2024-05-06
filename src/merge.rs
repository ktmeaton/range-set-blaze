use core::{iter::FusedIterator, ops::RangeInclusive};

use itertools::{Itertools, KMergeBy, MergeBy};

use crate::{integer::Integer, SortedDisjoint, SortedStarts};

/// Works with [`UnionIter`] to turn two [`SortedDisjoint`] iterators into a [`SortedDisjoint`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
///
/// Also see [`KMerge`].
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
    /// Creates a new [`Merge`] iterator from two [`SortedDisjoint`] iterators. See [`Merge`] for more details and examples.
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

/// Works with [`UnionIter`] to turn any number of [`SortedDisjoint`] iterators into a [`SortedDisjoint`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
///
/// Also see [`Merge`].
///
/// [`SortedDisjoint`]: crate::SortedDisjoint
/// [`UnionIter`]: crate::UnionIter

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
    /// Creates a new [`KMerge`] iterator from zero or more [`SortedDisjoint`] iterators. See [`KMerge`] for more details and examples.
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
