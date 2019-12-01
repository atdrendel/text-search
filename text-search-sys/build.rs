use std::{env, path::PathBuf};

fn main() {
  let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
    .canonicalize()
    .unwrap();
  let src = project_dir
    .join("vendor")
    .join("GNETextSearch")
    .join("GNETextSearch");

  build_gne_text_search(&src);
  generate_bindings(&project_dir, &src);
}

fn build_gne_text_search(src: &PathBuf) {
  cc::Build::new()
    .file(src.join("Set/countedset.c"))
    .file(src.join("String/stringbuf.c"))
    .file(src.join("Tree/ternarytree.c"))
    .file(src.join("UTF-8/tokenize.c"))
    .include(&src)
    .include(src.join("Set"))
    .include(src.join("String"))
    .compile("GNETextSearch");
}

fn generate_bindings(project_dir: &PathBuf, src: &PathBuf) {
  let header = string_from_path(src, Some("GNETextSearch.h"));
  let include_root = format!("-I{}", string_from_path(src, None));
  let include_set = format!("-I{}", string_from_path(src, Some("Set")));
  let include_tree = format!("-I{}", string_from_path(src, Some("Tree")));

  let bindings = bindgen::Builder::default()
    .header(header)
    .clang_arg(include_root)
    .clang_arg(include_set)
    .clang_arg(include_tree)
    .raw_line(
      "#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types, improper_ctypes)]",
    )
    .generate()
    .expect("Unable to generate bindings");

  let lib_rs_path = project_dir.join("src/lib.rs");
  bindings
    .write_to_file(lib_rs_path)
    .expect("Unable to write bindings");
}

fn string_from_path(root: &PathBuf, subpath: Option<&str>) -> String {
  let path: PathBuf;
  if let Some(subpath) = subpath {
    path = root.join(subpath);
  } else {
    path = root.to_path_buf();
  }
  path.into_os_string().into_string().unwrap()
}
