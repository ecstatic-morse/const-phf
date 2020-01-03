use crate::arr::ConstArray;
use crate::sig::MAX_KEYSIG_LEN;

#[derive(Clone, Copy, Debug)]
pub struct ByteMultiSet(ConstArray<Entry, MAX_KEYSIG_LEN>);

impl ByteMultiSet {
    pub const fn new() -> Self {
        ByteMultiSet(ConstArray::new(Entry::new(0, 0)))
    }

    pub const fn insert(&mut self, c: u8) {
        if let Some(entry) = self.entry_mut(c) {
            entry.count += 1;
            return;
        }

        self.0.push(Entry::new(c, 1));
    }

    pub const fn remove(&mut self, c: u8) {
        let mut i = 0;
        let len = self.0.len();
        let entries = self.0.as_slice_mut();
        let mut to_remove = None;

        while i < len {
            if entries[i].c == c {
                entries[i].count -= 1;
                if entries[i].count == 0 {
                    to_remove = Some(i);
                }
            }

            i += 1;
        }

        if let Some(i) = to_remove {
            self.0.swap_remove(i);
        }
    }

    pub const fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        iter!(ref entry in self.0.as_slice() => {
            if let Some(other_entry) = other.entry(entry.c) {
                if entry.count == other_entry.count {
                    continue;
                }
            }

            return false;
        });

        true
    }

    const fn entry(&self, c: u8) -> Option<&Entry> {
        iter!(ref entry in self.0.as_slice() => {
            if entry.c == c {
                return Some(entry)
            }
        });

        None
    }

    const fn entry_mut(&mut self, c: u8) -> Option<&mut Entry> {
        let mut i = 0;
        while i < self.0.len() {
            if self.0.as_slice()[i].c == c {
                return Some(&mut self.0.as_slice_mut()[i]);
            }

            i += 1;
        }

        None
    }
}

#[derive(Clone, Copy, Debug)]
struct Entry {
    c: u8,
    count: u8,
}

impl Entry {
    const fn new(c: u8, count: u8) -> Self {
        Entry { c, count }
    }
}

pub const fn rarest_char_in_disjoint_union(
    a: &ByteMultiSet,
    b: &ByteMultiSet,
    char_freq: &[u8; 256],
) -> Option<u8> {
    let mut rarest_char = None;

    // Handle elements only in `a` or in both `a` and `b`.
    iter!(ref entry in a.0.as_slice() => {
        if let Some(other_entry) = b.entry(entry.c) {
            if other_entry.count == entry.count {
                continue;
            }
        }

        let freq_to_beat = match rarest_char {
            None => u8::max_value(),
            Some(c) => char_freq[c as usize],
        };

        if char_freq[entry.c as usize] < freq_to_beat {
            rarest_char = Some(entry.c);
        }
    });

    // Handle elements only in `b`.
    iter!(ref entry in b.0.as_slice() => {
        if let Some(_) = a.entry(entry.c) {
            continue;
        }

        let freq_to_beat = match rarest_char {
            None => u8::max_value(),
            Some(c) => char_freq[c as usize],
        };

        if char_freq[entry.c as usize] < freq_to_beat {
            rarest_char = Some(entry.c);
        }
    });

    rarest_char
}
