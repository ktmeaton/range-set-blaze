use core::iter::FusedIterator;

use itertools::{Itertools, KMergeBy, MergeBy};

use crate::integer::Integer;
use crate::map::{CloneBorrow, ValueOwned};
use crate::range_values::SetPriorityMap;

use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap, SortedDisjointMap};

/// Works with [`UnionIter`] to turn any number of [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
///
/// Also see [`KMergeMap`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`UnionIter`]: crate::UnionIter
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MergeMap<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
{
    #[allow(clippy::type_complexity)]
    iter: MergeBy<
        SetPriorityMap<T, V, VR, L>,
        SetPriorityMap<T, V, VR, R>,
        fn(&Priority<T, V, VR>, &Priority<T, V, VR>) -> bool,
    >,
}

impl<T, V, VR, L, R> MergeMap<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
{
    /// Creates a new [`MergeMap`] iterator from two [`SortedDisjointMap`] iterators. See [`MergeMap`] for more details and examples.
    pub fn new(left: L, right: R) -> Self {
        let left = SetPriorityMap::new(left, 0);
        let right = SetPriorityMap::new(right, 1);
        Self {
            // We sort only by start -- priority is not used until later.
            iter: left.merge_by(right, |a, b| a.start() < b.start()),
        }
    }
}

impl<T, V, VR, L, R> FusedIterator for MergeMap<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
{
}

impl<T, V, VR, L, R> Iterator for MergeMap<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
{
    type Item = Priority<T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, V, VR, L, R> PrioritySortedStartsMap<T, V, VR> for MergeMap<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
{
}

/// Works with [`UnionIter`] to turn two [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
///
/// Also see [`MergeMap`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`UnionIter`]: crate::UnionIter
///
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct KMergeMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    #[allow(clippy::type_complexity)]
    iter:
        KMergeBy<SetPriorityMap<T, V, VR, I>, fn(&Priority<T, V, VR>, &Priority<T, V, VR>) -> bool>,
}

impl<T, V, VR, I> KMergeMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    /// Creates a new [`KMergeMap`] iterator from zero or more [`SortedDisjointMap`] iterators. See [`KMergeMap`] for more details and examples.
    pub fn new<K>(iter: K) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        // Prioritize from left to right
        let iter = iter.into_iter().enumerate().map(|(i, x)| {
            let priority_number = i;
            SetPriorityMap::new(x, priority_number)
        });
        // Merge RangeValues by start with ties broken by priority
        let iter: KMergeBy<
            SetPriorityMap<T, V, VR, I>,
            fn(&Priority<T, V, VR>, &Priority<T, V, VR>) -> bool,
        > = iter.kmerge_by(|a, b| {
            // We sort only by start -- priority is not used until later.
            a.start() < b.start()
        });
        Self { iter }
    }
}

impl<T, V, VR, I> FusedIterator for KMergeMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}

impl<T, V, VR, I> Iterator for KMergeMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,

    I: SortedDisjointMap<T, V, VR>,
{
    type Item = Priority<T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, V, VR, I> PrioritySortedStartsMap<T, V, VR> for KMergeMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}
