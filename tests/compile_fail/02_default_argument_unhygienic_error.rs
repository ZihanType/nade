pub use nade::base::*;

mod foo {
    use std::path::{Path, PathBuf};

    use nade::nade;

    #[nade]
    pub fn bar<P: AsRef<Path>>(#[nade(Path::new(".").canonicalize().unwrap())] p: P) -> PathBuf {
        p.as_ref().to_path_buf()
    }
}

use std::env;

use foo::bar;

fn main() {
    assert_eq!(bar!(), env::current_dir().unwrap());
}
