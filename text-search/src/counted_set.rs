use std::ptr::NonNull;
use text_search_sys::{
  tsearch_countedset, tsearch_countedset_add_int,
  tsearch_countedset_contains_int, tsearch_countedset_copy,
  tsearch_countedset_free, tsearch_countedset_get_count,
  tsearch_countedset_init, tsearch_countedset_intersect,
  tsearch_countedset_minus, tsearch_countedset_ptr,
  tsearch_countedset_remove_int, tsearch_countedset_union,
};

#[derive(Debug)]
pub struct CountedSet {
  raw: tsearch_countedset_ptr,
}

impl CountedSet {
  pub fn new() -> CountedSet {
    unsafe {
      CountedSet {
        raw: tsearch_countedset_init(),
      }
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
