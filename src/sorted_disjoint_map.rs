use crate::map::BitSubRangesMap;
use crate::range_values::RangeValuesIter;
use crate::range_values::RangeValuesToRangesIter;
use crate::sym_diff_iter_map::SymDiffIterMap;
use crate::BitOrMapMerge;
use crate::BitXorMapMerge;
use crate::DynSortedDisjointMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Debug;
use core::iter::FusedIterator;
use core::marker::PhantomData;
// use alloc::format;
// use alloc::string::String;
// use core::{
//     iter::FusedIterator,
//     ops::{self, RangeInclusive},
// };
use crate::map::BitAndRangesMap;
use crate::NotIter;
use std::ops;

use crate::intersection_iter_map::IntersectionIterMap;
use crate::map::CloneBorrow;
use crate::sorted_disjoint::SortedDisjoint;
use crate::{map::ValueOwned, union_iter_map::UnionIterMap, Integer, RangeMapBlaze};
use core::ops::RangeInclusive;

/// Internally, a trait used to mark iterators that provide ranges sorted by start, but not necessarily by end,
/// and may overlap.
#[doc(hidden)] // cmk don't hide so much stuff.ks
pub trait SortedStartsMap<T, V, VR>:
    Iterator<Item = (RangeInclusive<T>, VR)> + FusedIterator
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
}

/// This is sorted by starts and contains priority information, but it is not sorted by priority.
pub trait PrioritySortedStartsMap<T, V, VR>:
    Iterator<Item = Priority<T, V, VR>> + FusedIterator
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
}

/// This is sorted and disjoint and contains priority information, but it is not sorted by priority.
pub trait PrioritySortedDisjointMap<T, V, VR>: PrioritySortedStartsMap<T, V, VR>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
}

/// The trait used to mark iterators that provide ranges that are sorted by start and disjoint. Set operations on
/// iterators that implement this trait can be performed in linear time.
///
/// # Table of Contents
/// * [`SortedDisjointMap` Constructors](#SortedDisjointMap-constructors)
///   * [Examples](#constructor-examples)
/// * [`SortedDisjointMap` Set and Other Operations](#SortedDisjointMap-set-and-other-operations)
///   * [Performance](#performance)
///   * [Examples](#examples)
/// * [How to mark your type as `SortedDisjointMap`](#how-to-mark-your-type-as-SortedDisjointMap)
///   * [Example – Find the ordinal weekdays in September 2023](#example--find-the-ordinal-weekdays-in-september-2023)
///
/// # `SortedDisjointMap` Constructors
///
/// You'll usually construct a `SortedDisjointMap` iterator from a [`RangeMapBlaze`] or a [`CheckSortedDisjointMap`].
/// Here is a summary table, followed by [examples](#constructor-examples). You can also [define your own
/// `SortedDisjointMap`](#how-to-mark-your-type-as-SortedDisjointMap).
///
/// | Input type | Method |
/// |------------|--------|
/// | [`RangeMapBlaze`] | [`ranges`] |
/// | [`RangeMapBlaze`] | [`into_ranges`] |
/// | [`RangeMapBlaze`]'s [`RangesIter`] | [`clone`] |
/// | sorted & disjoint ranges | [`CheckSortedDisjointMap::new`] |
/// | `SortedDisjointMap` iterator | [itertools `tee`] |
/// | `SortedDisjointMap` iterator | [`crate::dyn_sorted_disjoint::DynSortedDisjointMap::new`] |
/// |  *your iterator type* | *[How to mark your type as `SortedDisjointMap`][1]* |
///
/// [`ranges`]: RangeMapBlaze::ranges
/// [`into_ranges`]: RangeMapBlaze::into_ranges
/// [`clone`]: crate::RangesIter::clone
/// [itertools `tee`]: https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.tee
/// [1]: #how-to-mark-your-type-as-SortedDisjointMap
/// [`RangesIter`]: crate::RangesIter
///
/// ## Constructor Examples
///
/// ```
/// use range_set_blaze::prelude::*;
/// use itertools::Itertools;
///
/// // RangeMapBlaze's .ranges(), .range().clone() and .into_ranges()
/// let r = RangeMapBlaze::from_iter([3, 2, 1, 100, 1]);
/// let a = r.ranges();
/// let b = a.clone();
/// assert!(a.to_string() == "1..=3, 100..=100");
/// assert!(b.to_string() == "1..=3, 100..=100");
/// //    'into_ranges' takes ownership of the 'RangeMapBlaze'
/// let a = RangeMapBlaze::from_iter([3, 2, 1, 100, 1]).into_ranges();
/// assert!(a.to_string() == "1..=3, 100..=100");
///
/// // CheckSortedDisjointMap -- unsorted or overlapping input ranges will cause a panic.
/// let a = CheckSortedDisjointMap::new([1..=3, 100..=100]);
/// assert!(a.to_string() == "1..=3, 100..=100");
///
/// // tee of a SortedDisjointMap iterator
/// let a = CheckSortedDisjointMap::new([1..=3, 100..=100]);
/// let (a, b) = a.tee();
/// assert!(a.to_string() == "1..=3, 100..=100");
/// assert!(b.to_string() == "1..=3, 100..=100");
///
/// // DynamicSortedDisjointMap of a SortedDisjointMap iterator
/// let a = CheckSortedDisjointMap::new([1..=3, 100..=100]);
/// let b = DynSortedDisjointMap::new(a);
/// assert!(b.to_string() == "1..=3, 100..=100");
/// ```
///
/// # `SortedDisjointMap` Set Operations
///
/// | Method | Operator | Multiway (same type) | Multiway (different types) |
/// |--------|----------|----------------------|----------------------------|
/// | `a.`[`union`]`(b)` | `a` &#124; `b` | `[a, b, c].`[`union`][crate::MultiwaySortedDisjointMap::union]`()` | [`crate::MultiwayRangeSetBlaze::union`]`!(a, b, c)` |
/// | `a.`[`intersection`]`(b)` | `a & b` | `[a, b, c].`[`intersection`][crate::MultiwaySortedDisjointMap::intersection]`()` | [`crate::MultiwayRangeSetBlaze::intersection`]`!(a, b, c)` |
/// | `a.`[`difference`]`(b)` | `a - b` |  |  |
/// | `a.`[`symmetric_difference`]`(b)` | `a ^ b` |  |  |
/// | `a.`[`complement`]`()` | `!a` |  |  |
///
///
/// ## Performance
///
/// Every operation is implemented as a single pass over the sorted & disjoint ranges, with minimal memory.
///
/// This is true even when applying multiple operations. The last example below demonstrates this.
///
/// ## Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a0 = RangeMapBlaze::from_iter([1..=2, 5..=100]);
/// let b0 = RangeMapBlaze::from_iter([2..=6]);
/// let c0 = RangeMapBlaze::from_iter([2..=2, 6..=200]);
///
/// // 'union' method and 'to_string' method
/// let (a, b) = (a0.ranges(), b0.ranges());
/// let result = a.union(b);
/// assert_eq!(result.to_string(), "1..=100");
///
/// // '|' operator and 'equal' method
/// let (a, b) = (a0.ranges(), b0.ranges());
/// let result = a | b;
/// assert!(result.equal(CheckSortedDisjointMap::new([1..=100])));
///
/// // multiway union of same type
/// let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
/// let result = [a, b, c].union();
/// assert_eq!(result.to_string(), "1..=200");
///
/// // multiway union of different types
/// let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
/// let result = union_dyn!(a, b, !c);
/// assert_eq!(result.to_string(), "-2147483648..=100, 201..=2147483647");
///
/// // Applying multiple operators makes only one pass through the inputs with minimal memory.
/// let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
/// let result = a - (b | c);
/// assert!(result.to_string() == "1..=1");
/// ```
///
/// # How to mark your type as `SortedDisjointMap`
///
/// To mark your iterator type as `SortedDisjointMap`, you implement the `SortedStartsMap` and `SortedDisjointMap` traits.
/// This is your promise to the compiler that your iterator will provide inclusive ranges that disjoint and sorted by start.
///
/// When you do this, your iterator will get access to the
/// efficient set operations methods, such as [`intersection`] and [`complement`]. The example below shows this.
///
/// > To use operators such as `&` and `!`, you must also implement the [`BitAnd`], [`Not`], etc. traits.
/// >
/// > If you want others to use your marked iterator type, reexport:
/// > `pub use range_set_blaze::{SortedDisjointMap, SortedStartsMap};`
///
/// [`BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
/// [`Not`]: https://doc.rust-lang.org/std/ops/trait.Not.html
/// [`intersection`]: SortedDisjointMap::intersection
/// [`complement`]: SortedDisjointMap::complement
/// [`union`]: SortedDisjointMap::union
/// [`symmetric_difference`]: SortedDisjointMap::symmetric_difference
/// [`difference`]: SortedDisjointMap::difference
/// [`to_string`]: SortedDisjointMap::to_string
/// [`equal`]: SortedDisjointMap::equal
/// [multiway_union]: crate::MultiwaySortedDisjointMap::union
/// [multiway_intersection]: crate::MultiwaySortedDisjointMap::intersection
///
/// ## Example -- Find the ordinal weekdays in September 2023
/// ```
/// use core::ops::RangeInclusive;
/// pub use range_set_blaze::{SortedDisjointMap, SortedStartsMap};
///
/// // Ordinal dates count January 1 as day 1, February 1 as day 32, etc.
/// struct OrdinalWeekends2023 {
///     next_range: RangeInclusive<i32>,
/// }
///
/// // We promise the compiler that our iterator will provide
/// // ranges that are sorted and disjoint.
/// impl SortedStartsMap<i32> for OrdinalWeekends2023 {}
/// impl SortedDisjointMap<i32> for OrdinalWeekends2023 {}
///
/// impl OrdinalWeekends2023 {
///     fn new() -> Self {
///         Self { next_range: 0..=1 }
///     }
/// }
/// impl Iterator for OrdinalWeekends2023 {
///     type Item = RangeInclusive<i32>;
///     fn next(&mut self) -> Option<Self::Item> {
///         let (start, end) = self.next_range.clone().into_inner();
///         if start > 365 {
///             None
///         } else {
///             self.next_range = (start + 7)..=(end + 7);
///             Some(start.max(1)..=end.min(365))
///         }
///     }
/// }
///
/// use range_set_blaze::prelude::*;
///
/// let weekends = OrdinalWeekends2023::new();
/// let september = CheckSortedDisjointMap::new([244..=273]);
/// let september_weekdays = september.intersection(weekends.complement());
/// assert_eq!(
///     september_weekdays.to_string(),
///     "244..=244, 247..=251, 254..=258, 261..=265, 268..=272"
/// );
/// ```
pub trait SortedDisjointMap<T, V, VR>: SortedStartsMap<T, V, VR>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    ///cmk
    #[inline]
    fn into_sorted_disjoint(self) -> RangeValuesToRangesIter<T, V, VR, Self>
    where
        Self: Sized,
    {
        RangeValuesToRangesIter::new(self)
    }
    // I think this is 'Sized' because will sometimes want to create a struct (e.g. BitOrIter) that contains a field of this type

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their union.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([1..=1]);
    /// let b = RangeMapBlaze::from_iter([2..=2]).into_ranges();
    /// let union = a.union(b);
    /// assert_eq!(union.to_string(), "1..=2");
    ///
    /// // Alternatively, we can use "|" because CheckSortedDisjointMap defines
    /// // ops::bitor as SortedDisjointMap::union.
    /// let a = CheckSortedDisjointMap::new([1..=1]);
    /// let b = RangeMapBlaze::from_iter([2..=2]).into_ranges();
    /// let union = a | b;
    /// assert_eq!(union.to_string(), "1..=2");
    /// ```
    #[inline]
    fn union<R>(self, other: R) -> BitOrMapMerge<T, V, VR, Self, R::IntoIter>
    where
        // cmk why must say SortedDisjointMap here by sorted_disjoint doesn't.
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, V, VR>,
        Self: Sized,
    {
        // cmk why this into iter stuff that is not used?
        UnionIterMap::new2(self, other.into_iter())
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their intersection.
    ///
    /// /// cmk Tell that right-and-side must be a set, not a map
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([2..=3]).into_ranges();
    /// let intersection = a.intersection(b);
    /// assert_eq!(intersection.to_string(), "2..=2");
    ///
    /// // Alternatively, we can use "&" because CheckSortedDisjointMap defines
    /// // ops::bitand as SortedDisjointMap::intersection.
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([2..=3]).into_ranges();
    /// let intersection = a & b;
    /// assert_eq!(intersection.to_string(), "2..=2");
    /// ```
    #[inline]
    fn intersection<R>(
        self,
        other: R,
    ) -> BitAndRangesMap<T, V, VR, Self, RangeValuesToRangesIter<T, V, VR, R::IntoIter>>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, V, VR>,
        Self: Sized,
    {
        let sorted_disjoint_map = other.into_iter();
        let sorted_disjoint = sorted_disjoint_map.into_sorted_disjoint();
        IntersectionIterMap::new(self, sorted_disjoint)
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their intersection.
    ///
    /// /// cmk Tell that right-and-side must be a set, not a map
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([2..=3]).into_ranges();
    /// let intersection = a.intersection(b);
    /// assert_eq!(intersection.to_string(), "2..=2");
    ///
    /// // Alternatively, we can use "&" because CheckSortedDisjointMap defines
    /// // ops::bitand as SortedDisjointMap::intersection.
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([2..=3]).into_ranges();
    /// let intersection = a & b;
    /// assert_eq!(intersection.to_string(), "2..=2");
    /// ```
    #[inline]
    fn intersection_with_set<R>(self, other: R) -> BitAndRangesMap<T, V, VR, Self, R::IntoIter>
    where
        R: IntoIterator<Item = RangeInclusive<T>>, // cmk0 is this bound needed?
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        let sorted_disjoint = other.into_iter();
        IntersectionIterMap::new(self, sorted_disjoint)
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their set difference.
    ///
    /// cmk Tell that right-and-side must be a set, not a map
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([2..=3]).into_ranges();
    /// let difference = a.difference(b);
    /// assert_eq!(difference.to_string(), "1..=1");
    ///
    /// // Alternatively, we can use "-" because CheckSortedDisjointMap defines
    /// // ops::sub as SortedDisjointMap::difference.
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([2..=3]).into_ranges();
    /// let difference = a - b;
    /// assert_eq!(difference.to_string(), "1..=1");
    /// ```
    #[inline]
    fn difference<R>(
        self,
        other: R,
    ) -> BitSubRangesMap<T, V, VR, Self, RangeValuesToRangesIter<T, V, VR, R::IntoIter>>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, V, VR>,
        Self: Sized,
    {
        let sorted_disjoint_map = other.into_iter();
        let complement = sorted_disjoint_map.complement_to_set();
        IntersectionIterMap::new(self, complement)
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their set difference.
    ///
    /// cmk Tell that right-and-side must be a set, not a map
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([2..=3]).into_ranges();
    /// let difference = a.difference(b);
    /// assert_eq!(difference.to_string(), "1..=1");
    ///
    /// // Alternatively, we can use "-" because CheckSortedDisjointMap defines
    /// // ops::sub as SortedDisjointMap::difference.
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([2..=3]).into_ranges();
    /// let difference = a - b;
    /// assert_eq!(difference.to_string(), "1..=1");
    /// ```
    #[inline]
    fn difference_with_set<R>(self, other: R) -> BitSubRangesMap<T, V, VR, Self, R::IntoIter>
    where
        R: IntoIterator<Item = RangeInclusive<T>>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        let sorted_disjoint = other.into_iter();
        let complement = sorted_disjoint.complement();
        IntersectionIterMap::new(self, complement)
    }

    /// cmk
    /// returns a set, not a map
    #[inline]
    fn complement_to_set(self) -> NotIter<T, RangeValuesToRangesIter<T, V, VR, Self>>
    where
        Self: Sized,
    {
        let sorted_disjoint = self.into_sorted_disjoint();
        sorted_disjoint.complement()
    }

    /// cmk
    /// returns a set, not a map
    #[inline]
    fn complement(self, v: &V) -> RangeToRangeValueIter<T, V, NotIter<T, impl SortedDisjoint<T>>>
    where
        Self: Sized,
    {
        let complement = self.complement_to_set();
        RangeToRangeValueIter::new(complement, v)
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator
    /// of their symmetric difference.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([2..=3]).into_ranges();
    /// let symmetric_difference = a.symmetric_difference(b);
    /// assert_eq!(symmetric_difference.to_string(), "1..=1, 3..=3");
    ///
    /// // Alternatively, we can use "^" because CheckSortedDisjointMap defines
    /// // ops::bitxor as SortedDisjointMap::symmetric_difference.
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([2..=3]).into_ranges();
    /// let symmetric_difference = a ^ b;
    /// assert_eq!(symmetric_difference.to_string(), "1..=1, 3..=3");
    /// ```
    #[inline]
    fn symmetric_difference<R>(self, other: R) -> BitXorMapMerge<T, V, VR, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, V, VR>,
        Self: Sized,
        VR: CloneBorrow<V>,
    {
        SymDiffIterMap::new2(self, other.into_iter())
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently tells if they are equal. Unlike most equality testing in Rust,
    /// this method takes ownership of the iterators and consumes them.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([1..=2]);
    /// let b = RangeMapBlaze::from_iter([1..=2]).into_ranges();
    /// assert!(a.equal(b));
    /// ```
    fn equal<R>(self, other: R) -> bool
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, V, VR>,
        Self: Sized,
    {
        use itertools::Itertools;

        self.zip_longest(other.into_iter()).all(|pair| {
            match pair {
                itertools::EitherOrBoth::Both(
                    (self_range, self_value),
                    (other_range, other_value),
                ) => {
                    // Place your custom equality logic here for matching elements
                    self_range == other_range && self_value.borrow() == other_value.borrow()
                }
                _ => false, // Handles the case where iterators are of different lengths
            }
        })
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut v = RangeMapBlaze::new();
    /// assert!(v.is_empty());
    /// v.insert(1);
    /// assert!(!v.is_empty());
    /// ```
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_empty(mut self) -> bool
    where
        Self: Sized,
    {
        self.next().is_none()
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e., `other` contains at least all the elements in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let sup = CheckSortedDisjointMap::new([1..=3]);
    /// let set: CheckSortedDisjointMap<i32, _> = [].into();
    /// assert_eq!(set.is_subset(sup), true);
    ///
    /// let sup = CheckSortedDisjointMap::new([1..=3]);
    /// let set = CheckSortedDisjointMap::new([2..=2]);
    /// assert_eq!(set.is_subset(sup), true);
    ///
    /// let sup = CheckSortedDisjointMap::new([1..=3]);
    /// let set = CheckSortedDisjointMap::new([2..=2, 4..=4]);
    /// assert_eq!(set.is_subset(sup), false);
    /// ```
    // #[must_use]
    // #[inline]
    // #[allow(clippy::wrong_self_convention)]
    // fn is_subset<R>(self, other: R) -> bool
    // where
    //     R: IntoIterator<Item = Self::Item>,
    //     R::IntoIter: SortedDisjointMap<'a, T, V, VR>,
    //     Self: Sized,
    // {
    //     self.difference(other).is_empty()
    // }

    /// Returns `true` if the set is a superset of another,
    /// i.e., `self` contains at least all the elements in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let sub = RangeMapBlaze::from_iter([1, 2]);
    /// let mut set = RangeMapBlaze::new();
    ///
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(0);
    /// set.insert(1);
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(2);
    /// assert_eq!(set.is_superset(&sub), true);
    /// ```
    // #[inline]
    // #[must_use]
    // #[allow(clippy::wrong_self_convention)]
    // fn is_superset<R>(self, other: R) -> bool
    // where
    //     R: IntoIterator<Item = Self::Item>,
    //     R::IntoIter: SortedDisjointMap<'a, T, V, VR>,
    //     Self: Sized,
    // {
    //     other.into_iter().is_subset(self)
    // }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let a = RangeMapBlaze::from_iter([1..=3]);
    /// let mut b = RangeMapBlaze::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(4);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(1);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```
    // #[must_use]
    // #[inline]
    // #[allow(clippy::wrong_self_convention)]
    // fn is_disjoint<R>(self, other: R) -> bool
    // where
    //     R: IntoIterator<Item = Self::Item>,
    //     R::IntoIter: SortedDisjointMap<'a, T, V, VR>,
    //     Self: Sized,
    // {
    //     self.intersection(other).is_empty()
    // }

    /// Create a [`RangeMapBlaze`] from a [`SortedDisjointMap`] iterator.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = RangeMapBlaze::from_sorted_disjoint(CheckSortedDisjointMap::new([-10..=-5, 1..=2]));
    /// let a1: RangeMapBlaze<i32> = CheckSortedDisjointMap::new([-10..=-5, 1..=2]).into_range_set_blaze();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
    /// ```
    // cmk000 does RangeSetBlaze need something like this?
    fn into_range_map_blaze(self) -> RangeMapBlaze<T, V>
    where
        Self: Sized,
        V: Clone,
    {
        RangeMapBlaze::from_sorted_disjoint_map(self)
    }
}

// /// Gives the [`SortedDisjointMap`] trait to any iterator of ranges. The iterator will panic
// /// if/when it finds that the ranges are not actually sorted and disjoint.
// ///
// /// # Performance
// ///
// /// All checking is done at runtime, but it should still be fast.
// ///
// /// # Example
// ///
// /// ```
// /// use range_set_blaze::prelude::*;
// ///
// /// let a = CheckSortedDisjointMap::new(vec![1..=2, 5..=100]);
// /// let b = CheckSortedDisjointMap::new([2..=6]);
// /// let union = a | b;
// /// assert_eq!(union.to_string(), "1..=100");
// /// ```
// ///
// /// Here the ranges are not sorted and disjoint, so the iterator will panic.
// ///```should_panic
// /// use range_set_blaze::prelude::*;
// ///
// /// let a = CheckSortedDisjointMap::new(vec![1..=2, 5..=100]);
// /// let b = CheckSortedDisjointMap::new([2..=6,-10..=-5]);
// /// let union = a | b;
// /// assert_eq!(union.to_string(), "1..=100");
// /// ```
// #[derive(Debug, Clone)]
// #[must_use = "iterators are lazy and do nothing unless consumed"]
// pub struct CheckSortedDisjointMap<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T, V>>,
// {
//     pub(crate) iter: I,
//     prev_end: Option<T, V>,
//     seen_none: bool,
// }

// impl<T: Integer, I> SortedDisjointMap<'a, T, V, VR> for CheckSortedDisjointMap<T, I> where
//     I: Iterator<Item = RangeInclusive<T, V>>
// {
// }
// impl<T: Integer, I> SortedStartsMap<T, V> for CheckSortedDisjointMap<T, I> where
//     I: Iterator<Item = RangeInclusive<T, V>>
// {
// }

// impl<T, I> CheckSortedDisjointMap<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T, V>>,
// {
//     /// Creates a new [`CheckSortedDisjointMap`] from an iterator of ranges. See [`CheckSortedDisjointMap`] for details and examples.
//     pub fn new(iter: I) -> Self {
//         CheckSortedDisjointMap {
//             iter,
//             prev_end: None,
//             seen_none: false,
//         }
//     }
// }

// impl<T, V> Default for CheckSortedDisjointMap<T,array::IntoIter<RangeInclusive<T, V>, 0>>
// where
//     T: Integer,
// {
//     // Default is an empty iterator.
//     fn default() -> Self {
//         Self::new([].into_iter())
//     }
// }

// impl<T, I> FusedIterator for CheckSortedDisjointMap<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T, V>> + FusedIterator,
// {
// }

// impl<T, I> Iterator for CheckSortedDisjointMap<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T, V>>,
// {
//     type Item = RangeInclusive<T, V>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let next = self.iter.next();

//         let Some(range) = next.as_ref() else {
//             self.seen_none = true;
//             return next;
//         };

//         assert!(
//             !self.seen_none,
//             "iterator cannot return Some after returning None"
//         );
//         let (start, end) = range.clone().into_inner();
//         assert!(start <= end, "start must be less or equal to end");
//         assert!(
//             end <= T::safe_max_value(),
//             "end must be less than or equal to safe_max_value"
//         );
//         if let Some(prev_end) = self.prev_end {
//             assert!(
//                 prev_end < T::safe_max_value() && prev_end + T::one() < start,
//                 "ranges must be disjoint"
//             );
//         }
//         self.prev_end = Some(end);

//         next
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }

// impl<T: Integer, const N: usize> From<[RangeInclusive<T, V>; N]>
//     for CheckSortedDisjointMap<T, array::IntoIter<RangeInclusive<T, V>, N>>
// {
//     /// You may create a [`CheckSortedDisjointMap`] from an array of integers.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use range_set_blaze::prelude::*;
//     ///
//     /// let a0 = CheckSortedDisjointMap::new([1..=3, 100..=100]);
//     /// let a1: CheckSortedDisjointMap<_,_> = [1..=3, 100..=100].into();
//     /// assert_eq!(a0.to_string(), "1..=3, 100..=100");
//     /// assert_eq!(a1.to_string(), "1..=3, 100..=100");
//     /// ```
//     fn from(arr: [RangeInclusive<T, V>; N]) -> Self {
//         let iter = arr.into_iter();
//         Self::new(iter)
//     }
// }

// impl<T: Integer, I> ops::Not for CheckSortedDisjointMap<T, I>
// where
//     I: Iterator<Item = RangeInclusive<T, V>>,
// {
//     type Output = NotIterMap<T, V, Self>;

//     fn not(self) -> Self::Output {
//         self.complement()
//     }
// }

// impl<T: Integer, R, L> ops::BitOr<R> for CheckSortedDisjointMap<T, L>
// where
//     L: Iterator<Item = RangeInclusive<T, V>>,
//     R: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = BitOrMergeMap<T, V, Self, R>;

//     fn bitor(self, other: R) -> Self::Output {
//         SortedDisjointMap::union(self, other)
//     }
// }

// impl<'a, T: Integer, V, VR, R, L> ops::BitAnd<R> for CheckSortedDisjointMap<T, V, VR, L>
// where
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     L: Iterator<Item = RangeInclusive<T, V>>,
//     R: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = BitAndMergeMap<T, V, Self, R>;

//     fn bitand(self, other: R) -> Self::Output {
//         SortedDisjointMap::intersection(self, other)
//     }
// }

// impl<T: Integer, R, L> ops::Sub<R> for CheckSortedDisjointMap<T, L>
// where
//     L: Iterator<Item = RangeInclusive<T, V>>,
//     R: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = BitSubMergeMap<T, V, Self, R>;

//     fn sub(self, other: R) -> Self::Output {
//         SortedDisjointMap::difference(self, other)
//     }
// }

// impl<T: Integer, R, L> ops::BitXor<R> for CheckSortedDisjointMap<T, L>
// where
//     L: Iterator<Item = RangeInclusive<T, V>>,
//     R: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = BitXOrTeeMap<T, V, Self, R>;

//     fn bitxor(self, other: R) -> Self::Output {
//         SortedDisjointMap::symmetric_difference(self, other)
//     }
// }

// cmk could this have a better name
pub trait DebugToString<T: Integer, V: ValueOwned, VR>
where
    VR: CloneBorrow<V>,
{
    fn to_string(self) -> String;
}

impl<T, V, VR, M> DebugToString<T, V, VR> for M
where
    T: Integer + Debug,
    V: ValueOwned + Debug,
    VR: CloneBorrow<V>,
    M: SortedDisjointMap<T, V, VR> + Sized,
{
    fn to_string(self) -> String {
        self.map(|range_value| {
            let (range, value) = range_value;
            format!("({:?}, {:?})", range, value.borrow())
        })
        .collect::<Vec<_>>()
        .join(", ")
    }
}

// cmk0
// impl<T: Integer, V: ValueOwned, VR, I: SortedStartsMap<T, V, VR>> SortedStartsMap<T, V, VR>
//     for Tee<I>
// where
//     VR: CloneBorrow<V> + Clone, // cmk is the clone a good idea?
// {
// }

// // If the inputs have sorted starts, the output is sorted and disjoint.
// impl<T: Integer, V: ValueOwned, VR, I: SortedStartsMap<T, V, VR>> SortedDisjointMap<T, V, VR>
//     for Tee<I>
// where
//     VR: CloneBorrow<V> + Clone, // cmk is the clone a good idea?
// {
// }

// impl<'a, T, V, VR> PartialEq for RangeValue<T, V, VR>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.range == other.range && self.1.borrow() == other.1.borrow()
//     }
// }

// // Implement `Eq` because `BinaryHeap` requires it.
// impl<'a, T, V, VR> Eq for RangeValue<T, V, VR>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
// {
// }

/// Gives any iterator of cmk implements the [`SortedDisjointMap`] trait without any checking.
// cmk0 why was this hidden? check for others#[doc(hidden)]
/// doc
pub struct CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    iter: I,
    seen_none: bool,
    previous: Option<(RangeInclusive<T>, VR)>,
    phantom_data: PhantomData<V>,
}

// define new
impl<T, V, VR, I> CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    /// cmk doc
    pub fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = (RangeInclusive<T>, VR), IntoIter = I>,
    {
        CheckSortedDisjointMap {
            iter: iter.into_iter(),
            seen_none: false,
            previous: None,
            phantom_data: PhantomData,
        }
    }
}

impl<T, V, VR, I, J> From<J> for CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
    J: IntoIterator<Item = (RangeInclusive<T>, VR), IntoIter = I>,
{
    fn from(iter: J) -> Self {
        CheckSortedDisjointMap::new(iter)
    }
}

impl<T, V, VR, I> Default for CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)> + Default,
{
    fn default() -> Self {
        // Utilize I::default() to satisfy the iterator requirement.
        Self::new(I::default())
    }
}
// implement fused
impl<T, V, VR, I> FusedIterator for CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
}

fn range_value_clone<T, V, VR>(range_value: &(RangeInclusive<T>, VR)) -> (RangeInclusive<T>, VR)
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    let (range, value) = range_value;
    (range.clone(), value.clone_borrow())
}

// implement iterator
impl<T, V, VR, I> Iterator for CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    type Item = (RangeInclusive<T>, VR);

    fn next(&mut self) -> Option<Self::Item> {
        let range_value = self.iter.next();
        let Some(range_value) = range_value else {
            self.seen_none = true;
            return None;
        };
        // cmk should test all these
        assert!(!self.seen_none, "A value must not be returned after None");
        let Some(previous) = self.previous.take() else {
            self.previous = Some(range_value_clone(&range_value));
            return Some(range_value);
        };

        let previous_end = *previous.0.end();
        let (start, end) = range_value.0.clone().into_inner();
        assert!(start <= end, "Start must be <= end.",);
        assert!(
            end <= T::safe_max_value(),
            "End must be <= T::safe_max_value()"
        );
        assert!(previous_end < start, "Ranges must be disjoint and sorted");
        if previous_end + T::one() == start {
            assert!(
                previous.1.borrow() != range_value.1.borrow(),
                "Touching ranges must have different values"
            );
        }
        self.previous = Some(range_value_clone(&range_value));
        Some(range_value_clone(&range_value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// // cmk00 check
// // cmk00 make Fused but don't require it

/// cmk doc
#[derive(Clone, Debug)]
pub struct Priority<T, V, VR>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    range_value: (RangeInclusive<T>, VR),
    priority_number: usize,
    phantom_data: PhantomData<V>,
}

// new
impl<T, V, VR> Priority<T, V, VR>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    /// cmk doc
    pub fn new(range_value: (RangeInclusive<T>, VR), priority_number: usize) -> Self {
        Self {
            range_value,
            priority_number,
            phantom_data: PhantomData,
        }
    }
}

impl<T, V, VR> Priority<T, V, VR>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    /// Returns the priority number.
    pub fn priority_number(&self) -> usize {
        self.priority_number
    }
    /// Returns a reference to `range_value`.
    pub fn range_value(&self) -> &(RangeInclusive<T>, VR) {
        &self.range_value
    }

    /// Updates `range_value` with the given value.
    pub fn set_range_value(&mut self, value: (RangeInclusive<T>, VR)) {
        self.range_value = value;
    }

    /// Consumes `Priority` and returns `range_value`.
    pub fn into_range_value(self) -> (RangeInclusive<T>, VR) {
        self.range_value
    }

    /// Updates the range part of `range_value`.
    pub fn set_range(&mut self, range: RangeInclusive<T>) {
        self.range_value.0 = range;
    }

    /// Consumes `Priority` and returns the range part of `range_value`.
    pub fn into_range(self) -> RangeInclusive<T> {
        self.range_value.0
    }

    /// Returns the start of the range.
    pub fn start(&self) -> T {
        *self.range_value.0.start()
    }

    /// Returns the end of the range.
    pub fn end(&self) -> T {
        *self.range_value.0.end()
    }

    /// Returns the start and end of the range. (Assuming direct access to start and end)
    pub fn start_and_end(&self) -> (T, T) {
        (
            (*self.range_value.0.start()).clone(),
            (*self.range_value.0.end()).clone(),
        )
    }

    /// Returns a reference to the value part of `range_value`.
    pub fn value(&self) -> &VR {
        &self.range_value.1
    }
}
// Implement `PartialEq` to allow comparison (needed for `Eq`).
impl<T, V, VR> PartialEq for Priority<T, V, VR>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    fn eq(&self, other: &Self) -> bool {
        let result_cmk = self.priority_number == other.priority_number;
        assert!(!result_cmk, "Don't expect identical priority numbers");
        result_cmk
    }
}

// Implement `Eq` because `BinaryHeap` requires it.
impl<'a, T, V, VR> Eq for Priority<T, V, VR>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
{
}

// Implement `Ord` so the heap knows how to compare elements.
impl<'a, T, V, VR> Ord for Priority<T, V, VR>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // smaller is better
        other.priority_number.cmp(&self.priority_number)
    }
}

// Implement `PartialOrd` to allow comparison (needed for `Ord`).
impl<'a, T, V, VR> PartialOrd for Priority<T, V, VR>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned,
    I: SortedDisjoint<T>,
{
    inner: I,
    value: &'a V,
    phantom: PhantomData<T>,
}

impl<'a, T, V, I> RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned,
    I: SortedDisjoint<T>,
{
    pub fn new(inner: I, value: &'a V) -> Self {
        Self {
            inner,
            value,
            phantom: PhantomData,
        }
    }
}

impl<T, V, I> FusedIterator for RangeToRangeValueIter<'_, T, V, I>
where
    T: Integer,
    V: ValueOwned,
    I: SortedDisjoint<T>,
{
}

impl<'a, T, V, I> Iterator for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned,
    I: SortedDisjoint<T>,
{
    type Item = (RangeInclusive<T>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|range| (range, self.value))
    }
}

// implements SortedDisjointMap
impl<'a, T, V, I> SortedStartsMap<T, V, &'a V> for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned,
    I: SortedDisjoint<T>,
{
}
impl<'a, T, V, I> SortedDisjointMap<T, V, &'a V> for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned,
    I: SortedDisjoint<T>,
{
}

pub trait AnythingGoesMap<'a, T: Integer, V: ValueOwned + 'a, VR: CloneBorrow<V> + 'a>:
    Iterator<Item = (RangeInclusive<T>, VR)>
{
}

impl<'a, T, V, VR, I> AnythingGoesMap<'a, T, V, VR> for I
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
}

macro_rules! impl_sorted_map_traits_and_ops {
    ($IterType:ty, $V:ty, $VR:ty, $($more_generics:tt)*) => {

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> SortedStartsMap<T, $V, $VR> for $IterType
        where
            T: Integer,
        {
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> SortedDisjointMap<T, $V, $VR> for $IterType
        where
            T: Integer,
        {
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> ops::Not for $IterType
        where
            T: Integer,
        {
            type Output = NotIter<T, RangeValuesToRangesIter<T, $V, $VR, Self>>;

            fn not(self) -> Self::Output {
                self.complement_to_set()
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitOr<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $V, $VR>,
        {
            type Output = BitOrMapMerge<T, $V, $VR, Self, R>;

            fn bitor(self, other: R) -> Self::Output {
                SortedDisjointMap::union(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::Sub<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $V, $VR>,
        {
            type Output = BitSubRangesMap<T, $V, $VR, Self, RangeValuesToRangesIter<T, $V, $VR, R>>;

            fn sub(self, other: R) -> Self::Output {
                SortedDisjointMap::difference(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitXor<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $V, $VR>,
        {
            type Output = BitXorMapMerge<T, $V, $VR, Self, R>;

            #[allow(clippy::suspicious_arithmetic_impl)]
            fn bitxor(self, other: R) -> Self::Output {
                SortedDisjointMap::symmetric_difference(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitAnd<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $V, $VR>,
        {
            type Output = BitAndRangesMap<T, $V, $VR, Self, RangeValuesToRangesIter<T, $V, $VR, R>>;

            fn bitand(self, other: R) -> Self::Output {
                SortedDisjointMap::intersection(self, other)
            }
        }

    }
}

// cmk CheckList: Be sure that these are all tested in 'test_every_sorted_disjoint_method'
impl_sorted_map_traits_and_ops!(CheckSortedDisjointMap<T, V, VR, I>, V, VR, V: ValueOwned, VR: CloneBorrow<V>, I: Iterator<Item = (RangeInclusive<T>,  VR)>);
impl_sorted_map_traits_and_ops!(UnionIterMap<T, V, VR, I>, V, VR, VR: CloneBorrow<V>, V: ValueOwned, I: PrioritySortedStartsMap<T, V, VR>);
impl_sorted_map_traits_and_ops!(IntersectionIterMap< T, V, VR, I0, I1>, V, VR, V: ValueOwned, VR: CloneBorrow<V>, I0: SortedDisjointMap<T, V, VR>, I1: SortedDisjoint<T>);
impl_sorted_map_traits_and_ops!(SymDiffIterMap<T, V, VR, I>, V, VR, VR: CloneBorrow<V>, V: ValueOwned, I: PrioritySortedStartsMap<T, V, VR>);
impl_sorted_map_traits_and_ops!(RangeValuesIter<'a, T, V>, V, &'a V, 'a, V: ValueOwned );
impl_sorted_map_traits_and_ops!(DynSortedDisjointMap<'a, T, V, VR>, V, VR, 'a, V: ValueOwned, VR: CloneBorrow<V>);
// cmk remove impl_sorted_map_traits_and_ops!(SortedDisjointToUnitMap<T, I>, (), &'static (), I: SortedDisjoint<T>);
// cmk RangeIter and IntoRangesIter
