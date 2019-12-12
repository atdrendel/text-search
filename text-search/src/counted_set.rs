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

  pub fn clear(&self) {
    unsafe { tsearch_countedset_remove_all_ints(self.raw).expect() }
  }

  pub fn count(&self) -> usize {
    unsafe { tsearch_countedset_get_count(self.raw) }
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
      if before <= 0 {
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
  fn length_of_counted_set() {}

  #[test]
  fn is_counted_set_empty() {}

  #[test]
  fn clear_counted_set() {}

  #[test]
  fn count_of_counted_set() {
    let mut set = CountedSet::new();
    set.insert(0);
    set.insert(0);
    set.insert(1);
    set.insert(2);
    assert_eq!(3, set.count());
  }

  #[test]
  fn count_of_value() {
    let mut set = CountedSet::new();
    set.insert(0);
    set.insert(0);
    set.insert(0);
    set.insert(1);
    set.insert(1);
    set.insert(2);

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
    set.insert(0);
    set.insert(0);
    assert_eq!(true, set.remove(0));
    assert_eq!(true, set.remove(0));
    assert_eq!(false, set.remove(0));
  }

  #[test]
  fn remove_all_from_counted_set() {
    let mut set = CountedSet::new();
    set.insert(0);
    set.insert(0);
    assert_eq!(true, set.remove_all(0));
    assert_eq!(false, set.remove_all(0));
  }
}
