use crate::alloc::string::ToString;
use crate::merge_map::KMergeMap;
use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap};
use crate::{BitOrMapKMerge, BitOrMapMerge, MergeMap, SortedDisjointMap};
use alloc::format;
use alloc::string::String;
use alloc::{collections::BinaryHeap, vec};
use core::cmp::min;
use core::iter::FusedIterator;
use core::ops::RangeInclusive;
use itertools::Itertools;

use crate::unsorted_disjoint_map::UnsortedPriorityDisjointMap;
use crate::{map::ValueOwned, Integer};
use crate::{
    map::{CloneBorrow, SortedStartsInVecMap},
    unsorted_disjoint_map::AssumePrioritySortedStartsMap,
};

/// Turns any number of [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges. Uses [`Merge`]
/// or [`KMerge`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`Merge`]: crate::Merge
/// [`KMerge`]: crate::KMerge
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{UnionIterMap, Merge, SortedDisjointMap, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = UnionIterMap::new2(a, b);
/// assert_eq!(union.to_string(), "1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = a | b;
/// assert_eq!(union.to_string(), "1..=100")
/// ```
// cmk #[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct UnionIterMap<T, V, VR, SS>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    SS: PrioritySortedStartsMap<T, V, VR>,
{
    iter: SS,
    next_item: Option<Priority<T, V, VR>>,
    workspace: BinaryHeap<Priority<T, V, VR>>,
    gather: Option<(RangeInclusive<T>, VR)>,
    ready_to_go: Option<(RangeInclusive<T>, VR)>,
}

impl<T, V, VR, I> Iterator for UnionIterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: PrioritySortedStartsMap<T, V, VR>,
{
    type Item = (RangeInclusive<T>, VR);

    fn next(&mut self) -> Option<(RangeInclusive<T>, VR)> {
        // Keep doing this until we have something to return.
        loop {
            if let Some(value) = self.ready_to_go.take() {
                // If ready_to_go is Some, return the value immediately.
                // println!("cmk output1 range {:?}", value.0);
                return Some(value);
            };

            // if self.next_item should go into the workspace, then put it there, get the next, next_item, and loop
            if let Some(next_item) = self.next_item.take() {
                let (next_start, next_end) = next_item.start_and_end();

                // If workspace is empty, just push the next item
                let Some(best) = self.workspace.peek() else {
                    // println!(
                    //     "cmk pushing self.next_item {:?} into empty workspace",
                    //     next_item.0
                    // );
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    // println!(
                    //     "cmk reading new self.next_item via .next() {:?}",
                    //     cmk_debug_string(&self.next_item)
                    // );
                    // println!("cmk return to top of the main processing loop");
                    continue; // return to top of the main processing loop
                };
                // LATER: Could add this special case: If next value is the same as best value and the ending is later, and the start overlaps/touches, then just extend the best value.
                if next_start == best.start() {
                    // Only push if the priority is better or the end is greater
                    if &next_item > best || next_end > best.end() {
                        // println!("cmk pushing next_item {:?} into workspace", next_item.0);
                        self.workspace.push(next_item);
                    } else {
                        // println!(
                        //     "cmk throwing away next_item {:?} because of priority and length",
                        //     next_item.0
                        // );
                    }
                    self.next_item = self.iter.next();
                    // println!(
                    //     "cmk .next() self.next_item {:?}",
                    //     cmk_debug_string(&self.next_item)
                    // );
                    // println!("cmk return to top of the main processing loop");
                    continue; // return to top of the main processing loop
                }

                // It does not go into the workspace, so just hold it and keep processing.
                // println!(
                //     "cmk new start, so hold self.next_item {:?} for later",
                //     next_item.0
                // );
                self.next_item = Some(next_item);
            }

            // If the workspace is empty, we are done.
            let Some(best) = self.workspace.peek() else {
                debug_assert!(self.next_item.is_none());
                debug_assert!(self.ready_to_go.is_none());
                let value = self.gather.take();
                // println!("cmk output2 range {:?}", cmk_debug_string(&value));

                return value;
            };

            // We buffer for output the best item up to the start of the next item (if any).

            // Find the start of the next item, if any.
            let next_end = if let Some(next_item) = self.next_item.as_ref() {
                // println!(
                //     "cmk start-less1 {:?} {:?}",
                //     next_item.0.start(),
                //     best.0.end()
                // );
                min(next_item.start() - T::one(), best.end())
                // println!("cmk min {:?}", m);
            } else {
                best.end()
            };

            // Add the front of best to the gather buffer.
            if let Some(mut gather) = self.gather.take() {
                if gather.1.borrow() == best.value().borrow()
                    && *gather.0.end() + T::one() == best.start()
                {
                    // if the gather is contiguous with the best, then merge them
                    gather.0 = *gather.0.start()..=next_end;
                    // println!(
                    //     "cmk merge gather {:?} best {:?} as {:?} -> {:?}",
                    //     gather.0,
                    //     best.0,
                    //     *best.0.start()..=next_end,
                    //     gather.0
                    // );
                    self.gather = Some(gather);
                } else {
                    // if the gather is not contiguous with the best, then output the gather and set the gather to the best
                    // println!(
                    //     "cmk new ready-to-go {:?}, new gather front of best {:?} as {:?}",
                    //     gather.0,
                    //     best.0,
                    //     *best.0.start()..=next_end
                    // );
                    self.ready_to_go = Some(gather);
                    self.gather = Some((best.start()..=next_end, best.value().clone_borrow()));
                }
            } else {
                // if there is no gather, then set the gather to the best
                // println!(
                //     "cmk no gather,  capture front of best {:?} as {:?}",
                //     best.0,
                //     *best.0.start()..=next_end
                // );
                self.gather = Some((best.start()..=next_end, best.value().clone_borrow()))
            };

            // We also update the workspace to removing any items that are completely covered by the new_start.
            // We also don't need to keep any items that have a lower priority and are shorter than the new best.
            let mut new_workspace = BinaryHeap::new();
            while let Some(item) = self.workspace.pop() {
                let mut item = item;
                if item.end() <= next_end {
                    // too short, don't keep
                    // println!("cmk too short, don't keep in workspace {:?}", item.0);
                    continue; // while loop
                }
                item.set_range(next_end + T::one()..=item.end());
                let Some(new_best) = new_workspace.peek() else {
                    // println!("cmk no workspace, so keep {:?}", item.0);
                    // new_workspace is empty, so keep
                    new_workspace.push(item);
                    continue; // while loop
                };
                if &item < new_best && item.end() <= new_best.end() {
                    // println!("cmk item is lower priority {:?} and shorter {:?} than best item {:?},{:?} in new workspace, so don't keep",
                    // item.priority, item.0, new_best.priority, new_best.0);
                    // not as good as new_best, and shorter, so don't keep
                    continue; // while loop
                }

                // higher priority or longer, so keep
                // println!("cmk item is higher priority {:?} or longer {:?} than best item {:?},{:?} in new workspace, so keep",
                // item.priority, item.0, new_best.priority, new_best.0);
                new_workspace.push(item);
            }
            self.workspace = new_workspace;
        } // end of main loop
    }
}

#[allow(dead_code)]
fn cmk_debug_string<'a, T, V, VR>(item: &Option<(RangeInclusive<T>, VR)>) -> String
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V> + 'a,
{
    if let Some(item) = item {
        format!("Some({:?})", item.0)
    } else {
        "None".to_string()
    }
}

impl<T, V, VR, I> UnionIterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: PrioritySortedStartsMap<T, V, VR>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`UnionIterMap`] from zero or more [`SortedStartsMap`] iterators. See [`UnionIterMap`] for more details and examples.
    pub fn new(mut iter: I) -> Self {
        let item = iter.next();
        Self {
            iter,
            next_item: item,
            workspace: BinaryHeap::new(),
            gather: None,
            ready_to_go: None,
        }
    }
}

impl<T, V, VR, L, R> BitOrMapMerge<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIterMap`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIterMap`] for more details and examples.
    pub fn new2(left: L, right: R) -> Self {
        let iter = MergeMap::new(left, right);
        Self::new(iter)
    }
}

/// cmk doc
impl<T, V, VR, J> BitOrMapKMerge<T, V, VR, J>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    J: SortedDisjointMap<T, V, VR>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIterMap`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIterMap`] for more details and examples.
    pub fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMergeMap::new(k);
        Self::new(iter)
    }
}

// from iter (T, &V) to UnionIterMap
impl<'a, T, V> FromIterator<(T, &'a V)>
    for UnionIterMap<T, V, &'a V, SortedStartsInVecMap<T, V, &'a V>>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, &'a V)>,
    {
        let iter = iter.into_iter().map(|(x, value)| (x..=x, value));
        UnionIterMap::from_iter(iter)
    }
}

// // from iter (RangeInclusive<T>, &V) to UnionIterMap
// impl<'a, T: Integer + 'a, V: ValueOwned + 'a> FromIterator<(RangeInclusive<T>, &'a V)>
//     for UnionIterMap<T, V, &'a V, SortedStartsInVecMap<T, V, &'a V>>
// {
//     fn from_iter<I>(iter: I) -> Self
//     where
//         I: IntoIterator<Item = (RangeInclusive<T>, &'a V)>,
//     {
//         let iter = iter.into_iter();
//         let iter = iter.map(|(range, value)| (range, value));
//         UnionIterMap::from_iter(iter)
//     }
// }

// cmk used?
#[allow(dead_code)]
type SortedRangeValueVec<T, V, VR> =
    AssumePrioritySortedStartsMap<T, V, VR, vec::IntoIter<(RangeInclusive<T>, VR)>>;

// cmk simplify the long types
// from iter (T, VR) to UnionIterMap
impl<T, V, VR> FromIterator<(RangeInclusive<T>, VR)>
    for UnionIterMap<T, V, VR, SortedStartsInVecMap<T, V, VR>>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, VR)>,
    {
        let iter = iter.into_iter();
        // let iter = iter.map(|x| {
        //     println!("cmk x.priority {:?}", x.priority);
        //     x
        // });
        let iter = UnsortedPriorityDisjointMap::new(iter);
        UnionIterMap::from(iter)
    }
}

// from from UnsortedDisjointMap to UnionIterMap
impl<T, V, VR, I> From<UnsortedPriorityDisjointMap<T, V, VR, I>>
    for UnionIterMap<T, V, VR, SortedStartsInVecMap<T, V, VR>>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    #[allow(clippy::clone_on_copy)]
    fn from(unsorted_disjoint: UnsortedPriorityDisjointMap<T, V, VR, I>) -> Self {
        let iter = unsorted_disjoint.sorted_by(|a, b| {
            // We sort only by start -- priority is not used until later.
            a.start().cmp(&b.start())
        });
        let iter = AssumePrioritySortedStartsMap::new(iter);
        Self::new(iter)
    }
}

// cmk0 test that every iterator (that can be) is FusedIterator
impl<T, V, VR, I> FusedIterator for UnionIterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: PrioritySortedStartsMap<T, V, VR> + FusedIterator,
{
}

// cmk
// impl<'a, T, V, VR, I> ops::Not for UnionIterMap<'a, T, V, VR, I>
// where
//     I: SortedStartsMap<T, V>,
// {
//     type Output = NotIterMap<T, V, Self>;

//     fn not(self) -> Self::Output {
//         self.complement()
//     }
// }

// impl<'a, T, V, VR, R, L> ops::BitOr<R> for UnionIterMap<'a, T, V, VR, L>
// where
//     T: Integer + 'a,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     L: SortedStartsMap<'a, T, V, VR>,
//     R: SortedDisjointMap<'a, T, V, VR> + 'a,
// {
//     type Output = BitOrMergeMap<'a, T, V, VR, Self, R>;

//     fn bitor(self, rhs: R) -> Self::Output {
//         // It might be fine to optimize to self.iter, but that would require
//         // also considering field 'range'
//         SortedDisjointMap::union(self, rhs)
//     }
// }

// impl<'a, T, V, VR, R, L> ops::Sub<R> for UnionIterMap<'a, T, V, VR, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitSubMergeMap<T, V, Self, R>;

//     fn sub(self, rhs: R) -> Self::Output {
//         SortedDisjointMap::difference(self, rhs)
//     }
// }

// impl<'a, T, V, VR, R, L> ops::BitXor<R> for UnionIterMap<'a, T, V, VR, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitXOrTeeMap<T, V, Self, R>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitxor(self, rhs: R) -> Self::Output {
//         SortedDisjointMap::symmetric_difference(self, rhs)
//     }
// }

// impl<'a, T, V, VR, R, L> ops::BitAnd<R> for UnionIterMap<'a, T, V, VR, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitAndMergeMap<T, V, Self, R>;

//     fn bitand(self, other: R) -> Self::Output {
//         SortedDisjointMap::intersection(self, other)
//     }
// }

// impl<'a, T: Integer + 'a, V: ValueOwned + 'a, const N: usize> From<[(T, V); N]>
//     for UnionIterMap<'a, T, V, &'a V, SortedStartsInVecMap<'a, T, V, &'a V>>
// {
//     fn from(arr: [(T, &'a V); N]) -> Self {
//         // Directly create an iterator from the array and map it as needed
//         arr.iter()
//             .map(|&(t, v)| (t, v)) // This is a simple identity map; adjust as needed for your actual transformation
//             .collect() // Collect into UnionIterMap, relying on FromIterator
//     }
// }
