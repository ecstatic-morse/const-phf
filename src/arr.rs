use std::mem::MaybeUninit;

#[derive(Clone, Copy, Debug)]
pub struct ConstArray<T: Copy, const N: usize> {
    data: MaybeUninit<[T; N]>,
    length: usize,
}

impl<T: Copy, const N: usize> ConstArray<T, N> {
    pub const fn new(val: T) -> Self {
        let mut ret = ConstArray {
            data: MaybeUninit::<[T; N]>::uninit(),
            length: 0,
        };

        // FIXME: This first call to `data_mut` is UB. We have no choice, since neither
        // `mem::zeroed` nor `ptr::write` is a `const fn`, and we can't do `[val; N]` yet.
        let data = ret.data_mut();
        let mut i = 0;
        while i < data.len() {
            data[i] = val;
            i += 1;
        }

        ret
    }

    const fn data_mut(&mut self) -> &mut [T; N] {
        unsafe {
            std::mem::transmute(&mut self.data)
        }
    }

    const fn data(&self) -> &[T; N] {
        unsafe {
            std::mem::transmute(&self.data)
        }
    }

    pub const fn len(&self) -> usize {
        self.length
    }

    pub const fn get(&self, i: usize) -> Option<&T> {
        if i < self.len() {
            Some(&self.data()[i])
        } else {
            None
        }
    }

    pub const fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        if i < self.len() {
            Some(&mut self.data_mut()[i])
        } else {
            None
        }
    }

    pub const fn index(&self, i: usize) -> &T {
        expect!(self.get(i), "Index out of bounds")
    }

    pub const fn index_mut(&mut self, i: usize) -> &mut T {
        expect!(self.get_mut(i), "Index out of bounds")
    }

    pub const fn push(&mut self, val: T) {
        assert!(self.len() < N);

        let len = self.length;
        self.data_mut()[len] = val;
        self.length += 1;
    }

    pub const fn insert(&mut self, index: usize, val: T) {
        assert!(self.len() < N);
        assert!(index < self.len());

        let mut i = self.len();
        while i > index {
            self.data_mut()[i] = self.data()[i - 1];
            i += 1;
        }

        self.length += 1;
        self.data_mut()[index] = val;
    }

    pub const fn pop(&mut self) -> T {
        assert!(self.len() > 0);

        let ret = self.data()[self.length];
        self.length -= 1;
        ret
    }

    pub const fn swap_remove(&mut self, i: usize) {
        assert!(i < self.len());

        self.data_mut()[i] = self.data()[self.length];
        self.length -= 1;
    }

    pub const fn remove(&mut self, mut i: usize) {
        assert!(i < self.len());

        while (i + 1) < self.len() {
            self.data_mut()[i] = self.data()[i + 1];
            i += 1;
        }

        self.length -= 1;
    }

    pub const fn as_slice(&self) -> &[T] {
        let data = self.data().as_ptr();
        unsafe { &*std::ptr::slice_from_raw_parts(data, self.length) }
    }

    pub const fn as_slice_mut(&mut self) -> &mut [T] {
        // FIXME: This should call `as_mut_ptr` when it is `const`.
        let data = self.data_mut() as *mut [T] as *mut T;
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data, self.length) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array() {
        const fn test() {
            let mut arr: ConstArray<_, 20> = ConstArray::new(0isize);
            arr.push(1);
            arr.push(2);
            arr.push(3);

            assert!(arr.len() == 3);
            assert!(arr.as_slice()[1] == 2);
        }

        test();
    }
}
