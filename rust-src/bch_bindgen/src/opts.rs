#[macro_export]
macro_rules! opt_set {
    ($opts:ident, $n:ident, $v:expr) => {
        bch_bindgen::paste! {
            $opts.$n = $v;
            $opts.[<set_ $n _defined>](1)
        }
    };
}

#[macro_export]
macro_rules! opt_defined {
    ($opts:ident, $n:ident) => {
        bch_bindgen::paste! {
            $opts.[< $n _defined>]()
        }
    };
}

#[macro_export]
macro_rules! opt_get {
    ($opts:ident, $n:ident) => {
        if bch_bindgen::opt_defined!($opts, $n) == 0 {
            bch_bindgen::paste! {
                unsafe {
                    bch_bindgen::bcachefs::bch2_opts_default.$n
                }
            }
        } else {
            bch_bindgen::paste! {
                $opts.$n
            }
        }
    };
}
