// Replacement for `for` loops.
macro_rules! iter {
    (($idx:ident, $elem:pat) in $arr:expr => $block:block) => {
        {
            let mut i = 0;
            let len = $arr.len();
            while i < len {
                let $elem = $arr[i];
                #[allow(unused)]
                let $idx = i;
                i += 1;

                $block
            }
        }
    };

    ($elem:pat in $arr:expr => $block:block) => {
        iter!((_i, $elem) in $arr => $block);
    };
}

pub type Key<'a> = &'a [u8];

pub const fn key_eq(a: Key<'_>, b: Key<'_>) -> bool {
    if a.len() != b.len() {
        return false;
    }

    iter!((i, a) in a => {
        if a != b[i] {
            return false;
        }
    });

    true
}

pub const fn swap<T: Copy>(a: &mut T, b: &mut T) {
    let tmp = *a;
    *a = *b;
    *b = tmp;
}

pub const fn slice_swap<T: Copy>(s: &mut [T], a: usize, b: usize) {
    let tmp = s[a];
    s[a] = s[b];
    s[b] = tmp;
}

macro_rules! expect {
    ($expr:expr, $msg:literal) => {
        match $expr {
            Some(x) => x,
            None => panic!($msg),
        }
    };

    ($expr:expr) => {
        expect!($expr, "`expect` called on `Option::None`")
    }
}

macro_rules! expect_ok {
    ($expr:expr, $msg:literal) => {
        match $expr {
            Ok(x) => x,
            Err(_) => panic!($msg),
        }
    };

    ($expr:expr) => {
        expect_ok!($expr, "`expect` called on `Result::Err`")
    }
}

#[derive(Clone, Copy)]
pub enum ControlFlow<T> {
    Break(T),
    Continue,
}

macro_rules! ret {
    ($expr:expr) => {
        match $expr {
            x @ ControlFlow::Break(_) => return x,
            ControlFlow::Continue => {}
        }
    }
}

macro_rules! for_each_char_in_keysig {
    ($key:expr, $sig:expr, |$c:ident| $block:block) => {
        {
            let ref key = $key;
            iter!(idx in $sig => {
                if let Some($c) = crate::sig::index(key, idx) {
                    $block
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_test() {
        let mut arr = [3, 1, 2, 0];
        sort_by_key!(&mut arr, |x| x);
        assert_eq!(arr, [0, 1, 2, 3]);
    }
}
