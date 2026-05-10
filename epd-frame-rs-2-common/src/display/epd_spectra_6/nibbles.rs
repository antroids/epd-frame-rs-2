use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::AddAssign;

pub type Nibble = u8;

pub struct Nibbles<S: AsMut<[u8]> + AsRef<[u8]>, E: Into<Nibble> + From<Nibble>> {
    data: S,
    _phantom: PhantomData<E>,
    len: usize,
}

impl<S: AsMut<[u8]> + AsRef<[u8]>, E: Into<Nibble> + From<Nibble>> Nibbles<S, E> {
    pub fn new(mut data: S, len: usize) -> Self {
        assert!(
            data.as_mut().len() * 2 >= len,
            "Nibbles underlying slice doesn't have enough space"
        );
        Self {
            data,
            _phantom: Default::default(),
            len,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> E {
        if index >= self.len {
            panic!("Index out of bounds");
        }
        let left = index % 2 == 0;
        let pair = self.data.as_ref()[index / 2];
        (if left { pair >> 4 } else { pair & 0x0F }).into()
    }

    pub fn set(&mut self, index: usize, value: E) {
        if index >= self.len {
            panic!("Index out of bounds");
        }
        let left = index % 2 == 0;
        let pair = &mut self.data.as_mut()[index / 2];
        *pair = if left {
            (*pair & 0x0F) | (value.into() << 4)
        } else {
            (*pair & 0xF0) | (value.into() & 0x0F)
        }
    }

    pub fn as_underlying_data(&self) -> &S {
        &self.data
    }
}

impl<E: Into<Nibble> + From<Nibble>> Nibbles<Vec<u8>, E> {
    pub fn allocate(value: u8, size: usize) -> Self {
        Self::new(vec![value + (value << 4); underlying_data_len(size)], size)
    }
}

pub struct NibblesIterator<'a, S: AsMut<[u8]> + AsRef<[u8]>, E: Into<Nibble> + From<Nibble>> {
    nibbles: &'a Nibbles<S, E>,
    index: usize,
}

pub const fn underlying_data_len(nibbles_len: usize) -> usize {
    (nibbles_len + 1) / 2
}

impl<'a, S: AsMut<[u8]> + AsRef<[u8]>, E: Into<Nibble> + From<Nibble>> Iterator
    for NibblesIterator<'a, S, E>
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.nibbles.len() {
            let result = Some(self.nibbles.get(self.index));
            self.index.add_assign(1);
            result
        } else {
            None
        }
    }
}

impl<'a, S: AsMut<[u8]> + AsRef<[u8]>, E: Into<Nibble> + From<Nibble>> IntoIterator
    for &'a Nibbles<S, E>
{
    type Item = E;
    type IntoIter = NibblesIterator<'a, S, E>;

    fn into_iter(self) -> Self::IntoIter {
        NibblesIterator {
            nibbles: self,
            index: 0,
        }
    }
}
