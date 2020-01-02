use text_search_sys::{
  _Result, tsearch_countedset_add_int, tsearch_countedset_contains_int,
  tsearch_countedset_copy, tsearch_countedset_copy_ints,
  tsearch_countedset_free, tsearch_countedset_get_count,
  tsearch_countedset_get_count_for_int, tsearch_countedset_init,
  tsearch_countedset_intersect, tsearch_countedset_minus,
  tsearch_countedset_ptr, tsearch_countedset_remove_all_ints,
  tsearch_countedset_remove_int, tsearch_countedset_union, GNEInteger,
};

// FIXME
// =====
//
// `CountedSet` is currently implemented by wrapping GNETextSearch's
// `tsearch_countedset`, which is a balanced binary tree stored in
// a single, contiguous buffer.
//
// 1. Replace this with a simple `BTreeHashMap` implementation.
// 2. Make the class generic
// 3. Add support for non-mutating set functions, while maintaining
//    backwards compatability.

/// A counted set of 64-bit integers implemented by wrapping
/// GNETextSearch's `tsearch_countedset`, which is a balanced binary
/// tree stored in a single, contiguous buffer.
///
/// `CountedSet`, like a standard set, only holds a single copy of each
/// 64-bit integer added to it. However, unlike a standard set,
/// `CountedSet` keeps track of the number of times each integer has
/// been added to it.
///
/// `CountedSet` also supports the some standard set operations:
/// intersect, minus, and union.
///
/// # Examples
///
/// ```
/// use text_search::counted_set::CountedSet;
///
/// let mut set = CountedSet::new();
///
/// // Add some integers.
/// set.insert(0);
/// set.insert(0);
/// set.insert(1);
///
/// // Check if the set contains a specific integer.
/// println!("{}", set.contains(0)); // prints "true"
/// println!("{}", set.contains(1)); // prints "true"
/// println!("{}", set.contains(2)); // prints "false"
///
/// // Check how many times a specific integer has been inserted.
/// println!("{}", set.get_count(0)); // prints "2"
/// println!("{}", set.get_count(1)); // prints "1"
/// println!("{}", set.get_count(2)); // prints "0"
/// ```
#[derive(Debug)]
pub struct CountedSet {
  raw: tsearch_countedset_ptr,
}

// TODO: Replace i64 with generic Hash and Eq
impl CountedSet {
  /// Creates an empty `CountedSet`.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  /// let set = CountedSet::new();
  /// ```
  pub fn new() -> CountedSet {
    CountedSet {
      raw: unsafe { tsearch_countedset_init() },
    }
  }

  /// Returns the number of elements in the set.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// assert_eq!(0, set.len());
  /// set.insert(1);
  /// assert_eq!(1, set.len());
  /// set.insert(1);
  /// assert_eq!(1, set.len());
  /// set.insert(2);
  /// assert_eq!(2, set.len());
  /// ```
  pub fn len(&self) -> usize {
    unsafe { tsearch_countedset_get_count(self.raw) }
  }

  /// Returns `true` if the set contains no elements.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// assert_eq!(true, set.is_empty());
  /// set.insert(1);
  /// assert_eq!(false, set.is_empty());
  /// ```
  pub fn is_empty(&self) -> bool {
    // TODO: Don't get the full count for this.
    let count = unsafe { tsearch_countedset_get_count(self.raw) };
    count == 0
  }

  /// Clears the set, removing all values.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// set.insert(1);
  /// set.clear();
  /// assert_eq!(true, set.is_empty());
  /// ```
  pub fn clear(&mut self) {
    unsafe { tsearch_countedset_remove_all_ints(self.raw).expect() }
  }

  /// Substracts the values in `other` from the set.
  ///
  /// If values in `other` have been added multiple times, the counts for
  /// equivalent values in the set will be subtracted by that amount.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// set.insert(1);
  /// set.insert(1);
  /// set.insert(1);
  /// set.insert(2);
  ///
  /// let mut other = CountedSet::new();
  /// other.insert(1);
  /// other.insert(1);
  /// other.insert(2);
  ///
  /// set.minus(&other);
  /// assert_eq!(1, set.get_count(1));
  /// assert_eq!(0, set.get_count(2));
  /// ```
  pub fn minus(&mut self, other: &CountedSet) {
    unsafe {
      tsearch_countedset_minus(self.raw, other.raw).expect();
    }
  }

  /// Adds the counts of the values in `other` to the set and removes from the
  /// set all values not also contained in `other`.
  ///
  /// If matching values in `other` have been added multiple times, the
  /// counts for equivanent values in the set will be increased by that
  /// amount.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// set.insert(1);
  /// set.insert(1);
  /// set.insert(1);
  /// set.insert(2);
  /// set.insert(3);
  ///
  /// let mut other = CountedSet::new();
  /// other.insert(1);
  /// other.insert(1);
  /// other.insert(2);
  /// other.insert(4);
  ///
  /// set.intersect(&other);
  /// assert_eq!(5, set.get_count(1));
  /// assert_eq!(2, set.get_count(2));
  /// assert_eq!(false, set.contains(3));
  /// assert_eq!(false, set.contains(4));
  /// ```
  pub fn intersect(&mut self, other: &CountedSet) {
    unsafe {
      tsearch_countedset_intersect(self.raw, other.raw).expect();
    }
  }

  /// Adds each value in `other` to the set.
  ///
  /// If matching values in `other` have been added multiple times, the
  /// counts for equivanent values in `self` will be increased by that
  /// amount.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// set.insert(1);
  /// set.insert(1);
  /// set.insert(1);
  /// set.insert(2);
  /// set.insert(3);
  ///
  /// let mut other = CountedSet::new();
  /// other.insert(1);
  /// other.insert(1);
  /// other.insert(2);
  /// other.insert(4);
  /// other.insert(4);
  ///
  /// set.union(&other);
  /// assert_eq!(5, set.get_count(1));
  /// assert_eq!(2, set.get_count(2));
  /// assert_eq!(1, set.get_count(3));
  /// assert_eq!(2, set.get_count(4));
  /// ```
  pub fn union(&mut self, other: &CountedSet) {
    unsafe {
      tsearch_countedset_union(self.raw, other.raw).expect();
    }
  }

  /// Returns `true` if the set contains the specified valued,
  /// otherwise `false`.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// set.insert(1);
  /// set.insert(2);
  /// set.insert(2);
  /// assert_eq!(true, set.contains(1));
  /// assert_eq!(true, set.contains(2));
  /// assert_eq!(false, set.contains(3));
  /// ```
  pub fn contains(&self, value: i64) -> bool {
    unsafe { tsearch_countedset_contains_int(self.raw, value) }
  }

  /// Returns the number of times the specified value has been added
  /// to the set.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// set.insert(1);
  /// set.insert(2);
  /// set.insert(2);
  /// assert_eq!(1, set.get_count(1));
  /// assert_eq!(2, set.get_count(2));
  /// assert_eq!(0, set.get_count(3));
  /// ```
  pub fn get_count(&self, value: i64) -> usize {
    unsafe { tsearch_countedset_get_count_for_int(self.raw, value) }
  }

  /// Adds a value to the set, returning the number of times the specified
  /// value has been added to the set.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// assert_eq!(1, set.insert(1));
  /// assert_eq!(2, set.insert(1));
  /// ```
  pub fn insert(&mut self, value: i64) -> usize {
    unsafe {
      tsearch_countedset_add_int(self.raw, value).expect();
      tsearch_countedset_get_count_for_int(self.raw, value)
    }
  }

  // TODO: Make sure to implement `tsearch_countedset_remove_int()` with
  // `remove_all()` because `tsearch_countedset_remove_int()` does **not**
  // have this method's behavior. It has the behavior of `remove_all()`.

  /// Decrements by one the count of the specified value in the
  /// set. Returns the new count of the specified value in the set.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// set.insert(1);
  /// set.insert(1);
  ///
  /// assert_eq!(1, set.remove(1));
  /// assert_eq!(0, set.remove(1));
  /// assert_eq!(0, set.remove(1));
  /// ```
  pub fn remove(&mut self, value: i64) -> usize {
    unsafe {
      let before = tsearch_countedset_get_count_for_int(self.raw, value);
      if before == 0 {
        0
      } else {
        let new_count = before - 1;
        tsearch_countedset_remove_int(self.raw, value).expect();
        for _ in 0..new_count {
          tsearch_countedset_add_int(self.raw, value);
        }
        new_count
      }
    }
  }

  /// Removes the specified value from the set, regardless of how
  /// many times it had been added to the set.
  ///
  /// Returns `true` if the value had been contained in the set,
  /// otherwise `false`.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// set.insert(1);
  /// set.insert(1);
  ///
  /// assert_eq!(true, set.remove_all(1));
  /// assert_eq!(false, set.remove_all(1));
  /// ```
  pub fn remove_all(&mut self, value: i64) -> bool {
    unsafe {
      let before = tsearch_countedset_get_count_for_int(self.raw, value);
      tsearch_countedset_remove_int(self.raw, value).expect();
      before > 0
    }
  }

  /// Copies the values contained in the set into a new `Vec`.
  ///
  /// The values in the returned `Vec` are sorted in decending order
  /// according to how many times the values were added to the set.
  ///
  /// The order of values added the same number of times to the set
  /// is undefined. So, if both `1` and `2` were each added three times
  /// to the set, the returned `Vec` could be either `vec![1, 2]` or
  /// `vec![2, 1]`.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// set.insert(1);
  /// set.insert(1);
  /// set.insert(2);
  /// set.insert(2);
  /// set.insert(2);
  /// set.insert(4);
  ///
  /// assert_eq!(vec![2, 1, 4], set.to_vec());
  /// ```
  pub fn to_vec(&self) -> Vec<i64> {
    let mut integers: Vec<GNEInteger> = Vec::with_capacity(0);
    let inout_count: Box<usize> = Box::new(0);

    unsafe {
      let integers_ptr: *mut *mut GNEInteger = &mut integers.as_mut_ptr();
      let inout_count_ptr = Box::into_raw(inout_count);
      tsearch_countedset_copy_ints(self.raw, integers_ptr, inout_count_ptr)
        .expect();
      let count = *Box::from_raw(inout_count_ptr);
      Vec::from_raw_parts(*integers_ptr, count, count)
    }
  }
}

impl Clone for CountedSet {
  /// Returns a copy of the set.
  ///
  /// # Examples
  ///
  /// ```
  /// use text_search::counted_set::CountedSet;
  ///
  /// let mut set = CountedSet::new();
  /// set.insert(1);
  /// set.insert(1);
  /// set.insert(2);
  ///
  /// let mut copy = set.clone();
  /// copy.insert(3);
  ///
  /// assert_eq!(copy.get_count(1), set.get_count(1));
  /// assert_eq!(copy.get_count(2), set.get_count(2));
  /// assert_ne!(copy.get_count(3), set.get_count(3));
  /// ```
  fn clone(&self) -> CountedSet {
    let raw = unsafe { tsearch_countedset_copy(self.raw) };
    assert!(raw.is_null() == false);
    CountedSet { raw }
  }
}

impl Drop for CountedSet {
  fn drop(&mut self) {
    unsafe {
      tsearch_countedset_free(self.raw);
    }
  }
}

trait _ResultExt {
  fn expect(self);
}

impl _ResultExt for _Result {
  fn expect(self) {
    match self {
      1 => {}
      _ => panic!(),
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;
  use std::ops::Range;

  #[test]
  fn length_of_counted_set() {
    let mut set = CountedSet::new();
    insert_integers(&mut set, vec![-1, 0, 0, 1, 2]);
    assert_eq!(4, set.len());
  }

  #[test]
  fn is_counted_set_empty() {
    let mut set = CountedSet::new();
    assert_eq!(true, set.is_empty());

    insert_integers(&mut set, vec![0, 0, 1, 2]);
    for int in vec![0, 1, 2] {
      set.remove(int);
      assert_eq!(false, set.is_empty());
    }
    set.remove(0);
    assert_eq!(true, set.is_empty());
  }

  #[test]
  fn clear_counted_set() {
    let mut set = CountedSet::new();
    insert_integers(&mut set, vec![91, 123456, -1]);
    assert_eq!(false, set.is_empty());
    set.clear();
    assert_eq!(true, set.is_empty());
  }

  #[test]
  fn minus_counted_set() {
    let mut set = CountedSet::new();
    let mut other = CountedSet::new();

    insert_integers(
      &mut set,
      vec![-999, 0, 999, -998, 12345, 999, 999, -1000, -12345],
    );
    insert_integers(&mut other, vec![0, -998, -1234, 999, -998, -1000, 1234]);

    set.minus(&other);

    assert_eq!(1, set.get_count(-999));
    assert_eq!(0, set.get_count(0));
    assert_eq!(2, set.get_count(999));
    assert_eq!(0, set.get_count(-998));
    assert_eq!(1, set.get_count(12345));
    assert_eq!(0, set.get_count(-1000));
    assert_eq!(1, set.get_count(-12345));
    assert_eq!(0, set.get_count(1234));
    assert_eq!(0, set.get_count(-1234));
  }

  #[test]
  fn intersect_counted_set() {
    let mut set = CountedSet::new();
    let mut other = CountedSet::new();

    insert_integers(&mut set, vec![-999, 0, 999, -998, 12345, -1000, -12345]);
    insert_integers(&mut other, vec![0, -998, -1234, 999, -998, -1000, 1234]);

    set.intersect(&other);

    assert_eq!(2, set.get_count(0));
    assert_eq!(3, set.get_count(-998));
    assert_eq!(2, set.get_count(999));
    assert_eq!(2, set.get_count(-1000));

    assert_eq!(0, set.get_count(-999));
    assert_eq!(0, set.get_count(12345));
    assert_eq!(0, set.get_count(-12345));
    assert_eq!(0, set.get_count(1234));
    assert_eq!(0, set.get_count(-1234));
  }

  #[test]
  fn union_counted_set() {
    let mut set = CountedSet::new();
    let mut other = CountedSet::new();

    insert_integers(&mut set, vec![-999, 0, 999, -998, 12345, -1000, -12345]);
    insert_integers(&mut other, vec![0, -998, -1234, 999, -998, -1000, 1234]);

    set.union(&other);

    assert_eq!(1, set.get_count(-999));
    assert_eq!(2, set.get_count(0));
    assert_eq!(3, set.get_count(-998));
    assert_eq!(2, set.get_count(999));
    assert_eq!(2, set.get_count(-1000));
    assert_eq!(1, set.get_count(12345));
    assert_eq!(1, set.get_count(-12345));
    assert_eq!(1, set.get_count(1234));
    assert_eq!(1, set.get_count(-1234));
  }

  #[test]
  fn contains_value() {
    let mut set = CountedSet::new();
    insert_integers(&mut set, vec![91, 123456, -1]);
    assert_eq!(true, set.contains(-1));
    assert_eq!(true, set.contains(91));
    assert_eq!(true, set.contains(123456));

    set.clear();
    assert_eq!(false, set.contains(-1));
    assert_eq!(false, set.contains(91));
    assert_eq!(false, set.contains(123456));
  }

  #[test]
  fn count_of_value() {
    let mut set = CountedSet::new();
    insert_integers(&mut set, vec![-1, -1, 0, 0, 0, 1, 1, 2]);
    assert_eq!(2, set.get_count(-1));
    assert_eq!(3, set.get_count(0));
    assert_eq!(2, set.get_count(1));
    assert_eq!(1, set.get_count(2));
  }

  #[test]
  fn insert_into_counted_set() {
    let mut set = CountedSet::new();
    assert_eq!(1, set.insert(1));
    assert_eq!(2, set.insert(1));
    assert_eq!(3, set.insert(1));
    assert_eq!(1, set.insert(2));
  }

  #[test]
  fn remove_from_counted_set() {
    let mut set = CountedSet::new();
    insert_integers(&mut set, vec![0, 0]);
    assert_eq!(1, set.remove(0));
    assert_eq!(0, set.remove(0));
    assert_eq!(0, set.remove(0));
  }

  #[test]
  fn remove_all_from_counted_set() {
    let mut set = CountedSet::new();
    insert_integers(&mut set, vec![0, 0]);
    assert_eq!(true, set.remove_all(0));
    assert_eq!(false, set.remove_all(0));
  }

  #[test]
  fn counted_set_to_vec() {
    let mut set = CountedSet::new();
    insert_integers(&mut set, vec![91, 91, 123456, -1, 91, -1]);
    let output = set.to_vec();
    assert_eq!(3, output.len());
    assert_eq!(91, output[0]);
    assert_eq!(-1, output[1]);
    assert_eq!(123456, output[2]);
  }

  #[test]
  fn large_counted_set_to_vec() {
    let mut set = CountedSet::new();

    let integer_map: HashMap<i64, usize> =
      [(-1, 4), (-9876, 3), (-12345, 1)].iter().cloned().collect();
    let range_map: HashMap<Range<i64>, usize> =
      [(0..1000, 2)].iter().cloned().collect();
    let mut vec: Vec<i64> = vec![];
    add_integers_from_map_to_vec(&mut vec, integer_map);
    add_integers_from_range_map_to_vec(&mut vec, range_map);
    let counted_set_length = vec.len() - 1005; // Duplicates are removed in CountedSet

    insert_integers(&mut set, vec);
    let output = set.to_vec();
    assert_eq!(counted_set_length, output.len());
    assert_eq!(-1, output[0]);
    assert_eq!(-9876, output[1]);
    assert_eq!(-12345, output[counted_set_length - 1]);
  }

  #[test]
  fn clone_counted_set() {
    let mut set = CountedSet::new();
    insert_integers(&mut set, vec![91, 91, 123456, 91, -1]);

    let copy = set.clone();

    set.clear();
    assert_eq!(true, set.is_empty());

    assert_eq!(3, copy.get_count(91));
    assert_eq!(1, copy.get_count(123456));
    assert_eq!(1, copy.get_count(-1));
  }

  fn add_integers_from_map_to_vec(
    vec: &mut Vec<i64>,
    map: HashMap<i64, usize>,
  ) {
    for (int, count) in map {
      for _ in 0..count {
        vec.push(int)
      }
    }
  }

  fn add_integers_from_range_map_to_vec(
    vec: &mut Vec<i64>,
    map: HashMap<Range<i64>, usize>,
  ) {
    for (range, count) in map {
      for int in range {
        for _ in 0..count {
          vec.push(int)
        }
      }
    }
  }

  fn insert_integers(counted_set: &mut CountedSet, integers: Vec<i64>) {
    for int in integers {
      counted_set.insert(int);
    }
  }
}
