pub use nade::base::*;

mod foo1 {
    use std::path::{Path, PathBuf};

    use nade::nade;

    #[nade]
    pub fn bar<P: AsRef<Path>>(#[nade(Path::new(".").canonicalize().unwrap())] p: P) -> PathBuf {
        p.as_ref().to_path_buf()
    }
}

mod foo2 {
    use std::path::{Path, PathBuf};

    use nade::nade;

    #[nade]
    pub fn bar<P: AsRef<Path>>(
        #[nade(::std::path::Path::new(".").canonicalize().unwrap())] p: P,
    ) -> PathBuf {
        p.as_ref().to_path_buf()
    }
}

mod foo3 {
    use std::path::{Path, PathBuf};

    use nade::nade;

    #[nade]
    pub fn bar<P: AsRef<Path>>(#[nade($crate::foo3::baz())] p: P) -> PathBuf {
        p.as_ref().to_path_buf()
    }

    pub fn baz() -> PathBuf {
        Path::new(".").canonicalize().unwrap()
    }
}

fn main() {
    {
        // import the required items into the current scope
        use std::path::Path;

        use foo1::bar;

        assert_eq!(bar!(), std::env::current_dir().unwrap());
    }

    {
        // when specifying default arguments, using crate full path
        use foo2::bar;

        assert_eq!(bar!(), std::env::current_dir().unwrap());
    }

    {
        // re-export items then using `$crate` to specify the full path
        use foo3::bar;

        assert_eq!(bar!(), std::env::current_dir().unwrap());
    }
}
