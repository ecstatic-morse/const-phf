//! Find unique signatures (subsets of characters) for each key.

use crate::util::{ControlFlow, Key};
use crate::set::ByteMultiSet;

pub const MAX_KEYSIG_LEN: usize = 7;

pub type Sig = crate::arr::ConstArray<isize, MAX_KEYSIG_LEN>;

pub const fn keysig(key: Key<'_>, sig: &[isize]) -> ByteMultiSet {
    let mut ret = ByteMultiSet::new();

    iter!(idx in sig => {
        if let Some(c) = index(key, idx) {
            ret.insert(c);
        }
    });

    ret
}

pub const fn index(key: Key<'_>, idx: isize) -> Option<u8> {
    let abs_idx = idx.abs() as usize;
    if abs_idx >= key.len() {
        return None;
    }

    if idx < 0 {
        Some(key[key.len() - abs_idx])
    } else {
        Some(key[abs_idx])
    }
}

pub const fn find_unique_keysig(keys: &[Key<'_>], start_len: usize) -> Result<Sig, ()> {
    let possible_indexes = &[0, 1, 2, 3, -1, -2, -3];
    let mut i = start_len;
    while i < possible_indexes.len() {
        if let Some(sig) = find_unique_keysig_with_len(possible_indexes, i, keys) {
            return Ok(sig);
        }

        i += 1;
    }

    Err(())
}

const fn find_unique_keysig_with_len(
    set: &[isize],
    k: usize,
    keys: &[Key<'_>],
) -> Option<Sig> {
    let mut keysig = Sig::new(0);
    match _comb(set, k, 0, &mut keysig, keys) {
        ControlFlow::Break(()) => Some(keysig),
        ControlFlow::Continue => None,
    }
}

const fn _comb(
    set: &[isize],
    k: usize,
    i: usize,
    chosen: &mut Sig,
    keys: &[Key<'_>],
) -> ControlFlow<()> {
    // Not enough items remain to choose `k`.
    if k > set.len() - i {
        return ControlFlow::Continue;
    }

    // All items have been chosen.
    if k == 0 {
        if is_keysig_unique(keys, chosen.as_slice()) {
            return ControlFlow::Break(());
        } else {
            return ControlFlow::Continue;
        }
    }

    // Recurse with the `i`-th element selected.
    chosen.push(set[i]);
    ret!(_comb(set, k - 1, i + 1, chosen, keys));

    // Recurse with the `i`-th element *not* selected.
    chosen.pop();
    ret!(_comb(set, k, i + 1, chosen, keys));

    ControlFlow::Continue
}

pub const fn is_keysig_unique(keys: &[Key<'_>], sig: &[isize]) -> bool {
    iter!((i, key) in keys => {
        let mut j = i + 1;
        while j < keys.len() {
            let other = keys[j];
            if key.len() == other.len() && keysig(key, sig).eq(&keysig(other, sig)) {
                return false;
            }

            j += 1;
        }
    });

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_keysig_test() {
        let keys: &[&[u8]] = &[
            b"as",
            b"break",
            b"const",
            b"continue",
            b"crate",
            b"else",
            b"enum",
            b"extern",
            b"false",
            b"fn",
            b"for",
            b"if",
            b"impl",
            b"in",
            b"let",
            b"loop",
            b"match",
            b"mod",
            b"move",
            b"mut",
            b"pub",
            b"ref",
            b"return",
            b"self",
            b"Self",
            b"static",
            b"struct",
            b"super",
            b"trait",
            b"true",
            b"type",
            b"unsafe",
            b"use",
            b"where",
            b"while",

            b"dyn",
            b"await",
            b"async",

            b"abstract",
            b"become",
            b"box",
            b"do",
            b"final",
            b"macro",
            b"override",
            b"priv",
            b"typeof",
            b"unsized",
            b"virtual",
            b"yield",

            b"try",
        ];

        assert!(find_unique_keysig(keys, 0).is_ok());
    }
}
