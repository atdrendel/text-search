use text_search_sys::{
  _Result, tsearch_countedset_add_int, tsearch_countedset_contains_int,
  tsearch_countedset_copy, tsearch_countedset_free,
  tsearch_countedset_get_count, tsearch_countedset_get_count_for_int,
  tsearch_countedset_init, tsearch_countedset_intersect,
  tsearch_countedset_minus, tsearch_countedset_ptr,
  tsearch_countedset_remove_all_ints, tsearch_countedset_remove_int,
  tsearch_countedset_union,
};

#[derive(Debug)]
pub struct CountedSet {
  raw: tsearch_countedset_ptr,
}

// TODO: Replace i64 with generic Hash and Eq
impl CountedSet {
  pub fn new() -> CountedSet {
    CountedSet {
      raw: unsafe { tsearch_countedset_init() },
    }
  }

  pub fn len(&self) -> usize {
    unsafe { tsearch_countedset_get_count(self.raw) }
  }

  pub fn is_empty(&self) -> bool {
    // TODO: Don't get the full count for this.
    let count = unsafe { tsearch_countedset_get_count(self.raw) };
    count == 0
  }

  pub fn clear(&mut self) {
    unsafe { tsearch_countedset_remove_all_ints(self.raw).expect() }
  }

  pub fn minus(&mut self, other: &CountedSet) {
    unsafe {
      tsearch_countedset_minus(self.raw, other.raw).expect();
    }
  }

  pub fn intersect(&mut self, other: &CountedSet) {
    unsafe {
      tsearch_countedset_intersect(self.raw, other.raw).expect();
    }
  }

  pub fn union(&mut self, other: &CountedSet) {
    unsafe {
      tsearch_countedset_union(self.raw, other.raw).expect();
    }
  }

  pub fn contains(&self, value: i64) -> bool {
    unsafe { tsearch_countedset_contains_int(self.raw, value) }
  }

  pub fn get_count(&self, value: i64) -> usize {
    unsafe { tsearch_countedset_get_count_for_int(self.raw, value) }
  }

  pub fn insert(&mut self, value: i64) -> bool {
    let count = unsafe {
      tsearch_countedset_add_int(self.raw, value).expect();
      tsearch_countedset_get_count_for_int(self.raw, value)
    };
    match count {
      0 | 1 => false,
      _ => true,
    }
  }

  // `tsearch_countedset_remove_int()` does **not** have this
  // method's behavior. It has the behavior of `remove_all()`.
  pub fn remove(&mut self, value: i64) -> bool {
    unsafe {
      let before = tsearch_countedset_get_count_for_int(self.raw, value);
      if before == 0 {
        false
      } else {
        tsearch_countedset_remove_int(self.raw, value).expect();
        for _ in 0..(before - 1) {
          tsearch_countedset_add_int(self.raw, value);
        }
        true
      }
    }
  }

  // `tsearch_countedset_remove_int()` has this method's behavior.
  pub fn remove_all(&mut self, value: i64) -> bool {
    unsafe {
      let before = tsearch_countedset_get_count_for_int(self.raw, value);
      tsearch_countedset_remove_int(self.raw, value).expect();
      before > 0
    }
  }
}

impl Clone for CountedSet {
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
    assert_eq!(false, set.insert(1));
    assert_eq!(true, set.insert(1));
    assert_eq!(true, set.insert(1));
    assert_eq!(false, set.insert(2));
  }

  #[test]
  fn remove_from_counted_set() {
    let mut set = CountedSet::new();
    insert_integers(&mut set, vec![0, 0]);
    assert_eq!(true, set.remove(0));
    assert_eq!(true, set.remove(0));
    assert_eq!(false, set.remove(0));
  }

  #[test]
  fn remove_all_from_counted_set() {
    let mut set = CountedSet::new();
    insert_integers(&mut set, vec![0, 0]);
    assert_eq!(true, set.remove_all(0));
    assert_eq!(false, set.remove_all(0));
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

  fn insert_integers(counted_set: &mut CountedSet, integers: Vec<i64>) {
    for int in integers {
      counted_set.insert(int);
    }
  }
}
