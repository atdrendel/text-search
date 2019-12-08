use text_search_sys;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn can_use_countedset() {
    let count = unsafe {
      let set = text_search_sys::tsearch_countedset_init();
      text_search_sys::tsearch_countedset_add_int(set, 1);
      text_search_sys::tsearch_countedset_add_int(set, 1);
      text_search_sys::tsearch_countedset_add_int(set, 2);
      let count = text_search_sys::tsearch_countedset_get_count(set);
      text_search_sys::tsearch_countedset_free(set);
      count
    };
    assert_eq!(2, count);
  }
}
