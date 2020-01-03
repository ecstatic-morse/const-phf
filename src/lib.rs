#![feature(const_fn)]
#![feature(const_if_match)]
#![feature(const_loop)]
#![feature(const_mut_refs)]
#![feature(const_panic)]
// Used only for `ConstArray`.
#![feature(const_generics)]
#![feature(const_transmute)]
#![feature(const_raw_ptr_deref)]
#![feature(slice_from_raw_parts)]
#![feature(const_slice_from_raw_parts)]
#![allow(incomplete_features)]

#[macro_use] mod util;

mod arr;
mod set;
mod sig;

use arr::ConstArray;
use set::{rarest_char_in_disjoint_union};
use sig::{Sig, find_unique_keysig, keysig, is_keysig_unique};
use util::{Key, key_eq};

const MAX_KEYS: usize = 255;
const SENTINEL: u8 = MAX_KEYS as u8;

const MAX_TABLE_SPARSITY: usize = 8;
const MAX_TABLE_LEN: usize = MAX_KEYS * MAX_TABLE_SPARSITY;

pub struct PhfMap<'a, T> {
    keys: &'a [(Key<'a>, T)],
    sig: Sig,

    table: [u8; MAX_TABLE_LEN],
    assoc_values: [u16; 256],
    max_hash: usize,
}

impl<'a, T> PhfMap<'a, T> {
    pub const fn new(kvs: &'a [(Key<'a>, T)]) -> Self {
        Self::with_keysig_len(kvs, 0)
    }

    pub const fn with_keysig_len(kvs: &'a [(Key<'a>, T)], len: usize) -> Self {
        let mut keys: ConstArray<Key<'a>, MAX_KEYS> = ConstArray::new(b"");
        iter!(ref kv in kvs => {
            keys.push(kv.0);
        });

        let sig = expect_ok!(find_unique_keysig(keys.as_slice(), len));

        Self::_with_keysig(kvs, sig)
    }

    pub const fn with_keysig(kvs: &'a [(Key<'a>, T)], sig: &[isize]) -> Self {
        let mut keysig = Sig::new(0);
        iter!(idx in sig => {
            keysig.push(idx);
        });

        Self::_with_keysig(kvs, keysig)
    }

    const fn _with_keysig(kvs: &'a [(Key<'a>, T)], sig: Sig) -> Self {
        assert!(kvs.len() <= MAX_KEYS);

        let mut keys: ConstArray<Key<'a>, MAX_KEYS> = ConstArray::new(b"");
        iter!(ref kv in kvs => {
            keys.push(kv.0);
        });
        let keys = keys.as_slice();

        let mut map = PhfMap {
            keys: kvs,
            sig,
            table: [SENTINEL; MAX_TABLE_LEN],
            assoc_values: [0; 256],
            max_hash: 0,
        };

        assert!(is_keysig_unique(keys, sig.as_slice()));

        let char_freq = count_char_frequency(keys, sig.as_slice());
        expect_ok!(find_assoc_values_random(&mut map, &char_freq));

        iter!(key in keys => {
            let hash = map.hash(key);
            if hash > map.max_hash {
                map.max_hash = hash;
            }
        });

        assert!(map.max_hash <= u16::max_value() as usize);

        let mut c = 0;
        while c <= u8::max_value() as usize {
            if char_freq[c] == 0 {
                map.assoc_values[c] = map.max_hash as u16;
            }

            c += 1;
        }

        map
    }

    const fn hash(&self, key: Key<'_>) -> usize {
        let mut hash = 0;

        iter!(idx in self.sig.as_slice() => {
            if let Some(c) = sig::index(key, idx) {
                hash += self.assoc_values[c as usize] as usize;
            }
        });

        hash += key.len();
        hash
    }

    pub const fn get(&self, key: Key<'_>) -> Option<&T> {
        let hash = self.hash(key);
        if hash > self.max_hash as usize {
            return None;
        }

        let idx = self.table[hash];
        if idx == SENTINEL {
            return None;
        }

        let (found, ref value) = self.keys[idx as usize];
        if !key_eq(key, found) {
            return None;
        }

        Some(value)
    }
}

const fn count_char_frequency(keys: &[Key<'_>], sig: &[isize]) -> [u8; 256] {
    let mut char_freq = [0; 256];

    iter!(ref key in keys => {
        iter!(idx in sig => {
            if let Some(c) = sig::index(key, idx) {
                char_freq[c as usize] += 1;
            }
        });
    });

    char_freq
}

const fn find_assoc_values_random<T>(
    map: &mut PhfMap<'_, T>,
    char_freq: &[u8; 256],
) -> Result<(), &'static str> {
    // Best by test.
    const INCREMENTS: [u16; 3] = [1, 3, 4];
    const MAX_TRIES: usize = 10_000;

    let mut n = 0;
    'retry: while n <= MAX_TRIES {
        n += 1;

        iter!((i, ref key) in map.keys => {
            let hash = map.hash(key.0);
            if hash >= map.table.len() {
                return Err("Failed to find perfect hash");
            }

            if map.table[hash] == SENTINEL {
                map.table[hash] = i as u8;
                continue;
            }

            // Collision with `other`
            let other = &map.keys[map.table[hash] as usize];
            let to_incr = rarest_char_in_disjoint_union(
                &keysig(key.0, map.sig.as_slice()),
                &keysig(other.0, map.sig.as_slice()),
                char_freq,
            );

            let to_incr = if let Some(c) = to_incr {
                c
            } else {
                return Err("duplicate keysigs");
            };

            // Clear the table
            map.table = [SENTINEL; MAX_TABLE_LEN];

            // Update associated values array
            let incr = INCREMENTS[n % INCREMENTS.len()];
            map.assoc_values[to_incr as usize] += incr;
            continue 'retry;
        });

        return Ok(());
    }

    Err("Failed to find perfect hash")
}

const fn freq_score(key: Key<'_>, sig: &[isize], char_freq: &[u8; 256]) -> isize {
    let mut ret = 0;
    for_each_char_in_keysig!(key, sig, |c| {
        ret += char_freq[c as usize] as isize;
    });

    ret
}

/*
const fn order_keywords(
    keys: &mut [Key<'_>],
    sig: &[isize],
    char_freq: &[u8; 256],
) {
    // Sort keys containing more commonly used characters near the start
    sort_by_key!(keys, |key| -freq_score(key, sig, char_freq));

    let mut settled_chars = [false; 256];

    let mut i = 0;
    while i < keys.len() {
        // Mark all characters in `keys[i]` as settled
        for_each_char_in_keysig!(keys[i], sig, |c| {
            settled_chars[c as usize] = true;
        });

        i += 1;

        // Look for keys whose keysig consists of all settled characters
        let mut j = i;
        while j < keys.len() {
            if !is_keysig_settled(keys[j], sig, &settled_chars) {
                continue;
            }

            // Found one at index `j`.
            let to_move = keys[j];
            keys.remove(j);
            keys.insert(i, to_move);
            i += 1;
            j += 1;
        }
    }
}

const fn is_keysig_settled(key: Key<'_>, sig: &[isize], settled_chars: &[bool; 256]) -> bool {
    for_each_char_in_keysig!(key, sig, |c| {
        if !settled_chars[c as usize] {
            return false;
        }
    });

    true
}

const fn find_assoc_values(
    keys: &[Key<'a>],
    sig: &[isize],
    table: &mut [u8; MAX_TABLE_LEN],
    assoc_values: &mut [u16; 256],
    settled: &mut [u8; 256],
    i: usize,
) -> Result<(), &'static str> {
    let max_assoc_value = keys.len();
    let key = keys[i];

    loop {
        for_each_char_in_keysig!(key, sig, |c| {
            assoc_values[c] += 1;
            if assoc_values[c] > max_assoc_value {
                return Err(());
            }

            let mut j = i + 1;
            while j < keys.len() {
                if


                let hash = hash(keys[j], sig, assoc_values);
                if hash > table.len() {
                    return Err(());
                }

                if table[hash] != SENTINEL {
                    return Err(());
                }

                table[hash] = j as u8;
            }
        });
    }

    Ok(())
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        const PHF: PhfMap<'static, usize> = PhfMap::new(&[
            (b"as", 1),
            (b"break", 2),
            (b"const", 3),
            (b"continue", 4),
            (b"crate", 5),
            (b"else", 6),
            (b"enum", 7),
            (b"extern", 8),
            (b"false", 9),
            (b"fn", 10),
            (b"for", 11),
            (b"if", 12),
            (b"impl", 13),
            (b"in", 14),
            (b"let", 15),
            (b"loop", 16),
            (b"match", 17),
            (b"mod", 18),
            (b"move", 19),
            (b"mut", 20),
            (b"pub", 21),
            (b"ref", 22),
            (b"return", 23),
            (b"self", 24),
            (b"Self", 25),
            (b"static", 26),
            (b"struct", 27),
            (b"super", 28),
            (b"trait", 29),
            (b"true", 30),
            (b"type", 31),
            (b"unsafe", 32),
            (b"use", 33),
            (b"where", 34),
            (b"while", 35),

            (b"dyn", 36),
            (b"await", 37),
            (b"async", 38),

            (b"abstract", 39),
            (b"become", 40),
            (b"box", 41),
            (b"do", 42),
            (b"final", 43),
            (b"macro", 44),
            (b"override", 45),
            (b"priv", 46),
            (b"typeof", 47),
            (b"unsized", 48),
            (b"virtual", 49),
            (b"yield", 50),

            (b"try", 51),
        ]);

        assert_eq!(PHF.get(b"while").copied(), Some(35));
        assert_eq!(PHF.get(b"return").copied(), Some(23));
        assert_eq!(PHF.get(b"whoo").copied(), None);
        assert_eq!(PHF.get(b"tipe").copied(), None);
    }
}
