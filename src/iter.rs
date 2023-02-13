use super::LimitedVec;

pub struct Iter<'a, T, const N: usize> {
    lvec: &'a LimitedVec<T, N>,
    pos: usize,
}

impl<'a, T, const N: usize> Iter<'a, T, N> {
    #[inline]
    pub fn new(lvec: &'a LimitedVec<T, N>) -> Self {
        Self { lvec, pos: 0 }
    }
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.lvec.get(self.pos)?;
        self.pos += 1;
        Some(item)
    }
}
