use crate::{
    merge::KMerge, Integer, Merge, SortedDisjoint, SortedStarts, SymDiffIterKMerge,
    SymDiffIterMerge,
};
use core::{
    cmp::{self, min},
    iter::FusedIterator,
    ops::RangeInclusive,
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
/// use range_set_blaze::{SymDiffIter, Merge, SortedDisjointMap, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = SymDiffIter::new(Merge::new(a, b));
/// assert_eq!(union.to_string(), "1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = a | b;
/// assert_eq!(union.to_string(), "1..=100")
/// ```
// cmk #[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    iter: I,
    next_item: Option<RangeInclusive<T>>,
    workspace: Vec<RangeInclusive<T>>,
    workspace_next_end: Option<T>,
    gather: Option<RangeInclusive<T>>,
    ready_to_go: Option<RangeInclusive<T>>,
}

fn min_next_end<T>(next_end: &Option<T>, next_item_end: T) -> Option<T>
where
    T: Integer,
{
    Some(next_end.map_or_else(
        || next_item_end,
        |current_end| cmp::min(current_end, next_item_end),
    ))
}

impl<T, I> FusedIterator for SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
}

// cmk0000 review this for simplifications
impl<T, I> Iterator for SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<RangeInclusive<T>> {
        // Keep doing this until we have something to return.
        loop {
            if let Some(value) = self.ready_to_go.take() {
                // If ready_to_go is Some, return the value immediately.
                // println!("cmk output1 range {:?}", value.0);
                return Some(value);
            };

            // if self.next_item should go into the workspace, then put it there, get the next, next_item, and loop
            if let Some(next_item) = self.next_item.take() {
                let (next_start, next_end) = next_item.clone().into_inner();

                // If workspace is empty, just push the next item
                let Some(best) = self.workspace.first() else {
                    // println!(
                    //     "cmk pushing self.next_item {:?} into empty workspace",
                    //     next_item.0
                    // );
                    self.workspace_next_end = min_next_end(&self.workspace_next_end, next_end);
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    // println!(
                    //     "cmk reading new self.next_item via .next() {:?}",
                    //     cmk_debug_string(&self.next_item)
                    // );
                    // println!("cmk return to top of the main processing loop");
                    continue; // return to top of the main processing loop
                };
                if next_start == *best.start() {
                    // Always push (this differs from UnionIterMap)
                    self.workspace_next_end = min_next_end(&self.workspace_next_end, next_end);
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
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
            let Some(best) = self.workspace.first() else {
                debug_assert!(self.next_item.is_none());
                debug_assert!(self.ready_to_go.is_none());
                let value = self.gather.take();
                // println!("cmk output2 range {:?}", cmk_debug_string(&value));

                return value;
            };

            // We buffer for output the best item up to the start of the next item (if any).

            // Find the start of the next item, if any.
            // unwrap() is safe because we know the workspace is not empty
            let mut next_end = self.workspace_next_end.take().unwrap();
            if let Some(next_item) = self.next_item.as_ref() {
                next_end = min(*next_item.start() - T::one(), next_end);
            }

            // Add the front of best to the gather buffer.
            if let Some(mut gather) = self.gather.take() {
                if *gather.end() + T::one() == *best.start() {
                    if self.workspace.len() % 2 == 1 {
                        // if the gather is contiguous with the best, then merge them
                        gather = *gather.start()..=next_end;
                        // println!(
                        //     "cmk merge gather {:?} best {:?} as {:?} -> {:?}",
                        //     gather.0,
                        //     best.0,
                        //     *best.0.start()..=next_end,
                        //     gather.0
                        // );
                        self.gather = Some(gather);
                    } else {
                        // if an even number of items in the workspace, then flush the gather
                        self.ready_to_go = Some(gather);
                        debug_assert!(self.gather.is_none());
                    }
                } else {
                    // if the gather is not contiguous with the best, then output the gather and set the gather to the best
                    // println!(
                    //     "cmk new ready-to-go {:?}, new gather front of best {:?} as {:?}",
                    //     gather.0,
                    //     best.0,
                    //     *best.0.start()..=next_end
                    // );
                    self.ready_to_go = Some(gather);
                    // cmk this code appear twice
                    if self.workspace.len() % 2 == 1 {
                        self.gather = Some(*best.start()..=next_end);
                    } else {
                        debug_assert!(self.gather.is_none());
                    }
                }
            } else {
                // if there is no gather, then set the gather to the best
                // println!(
                //     "cmk no gather,  capture front of best {:?} as {:?}",
                //     best.0,
                //     *best.0.start()..=next_end
                // );
                if self.workspace.len() % 2 == 1 {
                    self.gather = Some(*best.start()..=next_end);
                } else {
                    debug_assert!(self.gather.is_none());
                }
            };

            // We also update the workspace to removing any items that are completely covered by the new_start.
            // (Unlike UnionIterMap, we must keep any items that have a lower priority and are shorter than the new best.)
            // cmk use .filter() ?
            let mut new_workspace = Vec::new();
            let mut new_next_end = None;
            while let Some(item) = self.workspace.pop() {
                let mut item = item;
                if *item.end() <= next_end {
                    // too short, don't keep
                    // println!("cmk too short, don't keep in workspace {:?}", item.0);
                    continue; // while loop
                }
                item = next_end + T::one()..=*item.end();
                new_next_end = min_next_end(&new_next_end, *item.end());
                new_workspace.push(item);
            }
            self.workspace = new_workspace;
            self.workspace_next_end = new_next_end;
        } // end of main loop
    }
}

// #[allow(dead_code)]
// fn cmk_debug_string<'a, T, V, VR>(item: &Option<RangeInclusive<T>>) -> String
// where
//     T: Integer,
//     V: ValueOwned,
//     VR: CloneBorrow<V> + 'a,
// {
//     if let Some(item) = item {
//         format!("Some({:?})", item.0)
//     } else {
//         "None".to_string()
//     }
// }

impl<T, L, R> SymDiffIterMerge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIter`] for more details and examples.
    pub fn new2(left: L, right: R) -> Self {
        let iter = Merge::new(left, right);
        Self::new(iter)
    }
}

/// cmk doc
impl<T, J> SymDiffIterKMerge<T, J>
where
    T: Integer,
    J: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIter`] for more details and examples.
    pub fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMerge::new(k);
        Self::new(iter)
    }
}

impl<T, I> SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjointMap`] iterators.
    /// See [`SymDiffIter`] for more details and examples.
    pub fn new(mut iter: I) -> Self {
        let item = iter.next();
        Self {
            iter,
            next_item: item,
            workspace: Vec::new(),
            workspace_next_end: None,
            gather: None,
            ready_to_go: None,
        }
    }
}
