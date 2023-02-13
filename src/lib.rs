pub mod iter;

use core::ops::Index;
use iter::Iter;

#[cfg(feature = "with_serde")]
use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, PartialEq, Eq)]
pub struct LimitedVec<T, const N: usize>([Option<T>; N]);

impl<T, const N: usize> LimitedVec<T, N>
where
    T: Default + Copy,
{
    /// Creates a new limited vector
    #[inline]
    pub fn new() -> Self {
        LimitedVec([None; N])
    }
}

impl<T, const N: usize> LimitedVec<T, N> {
    /// Pushes a new value onto the LimitedVec
    #[inline]
    pub fn push(&mut self, item: T) {
        match self.next_mut() {
            Some(m) => *m = Some(item),
            None => panic!("Trying to push more elements than SmallVec can hold"),
        }
    }

    /// Pops the last element and return
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        let last_idx = self.last_idx()?;
        std::mem::replace(&mut self.0[last_idx], None)
    }

    /// Returns the count of items the vector is holding
    pub fn len(&self) -> usize {
        self.0.iter().take_while(|i| i.is_some()).count()
    }

    /// Returns `true` if there is no item pushed onto the LimitedVec
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the amount of items the LimitedVec can hold
    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    /// Returns the amount of free slots which can be used to push more items
    pub fn free(&self) -> usize {
        self.0.iter().rev().take_while(|i| i.is_none()).count()
    }

    /// Returns `true` if there is no free slot left
    #[inline]
    pub fn is_full(&self) -> bool {
        self.free() == 0
    }

    /// Gets an item by its index
    #[inline]
    pub fn get(&self, pos: usize) -> Option<&T> {
        if pos >= self.len() {
            return None;
        }
        Some(&self[pos])
    }

    /// Returns the last item of the LimitedVec or None if its empty.
    pub fn last_mut(&mut self) -> Option<&mut T> {
        let last_pos = self.last_idx()?;
        self.0[last_pos].as_mut()
    }

    /// Returns the last item of the LimitedVec or None if its empty.
    pub fn last(&self) -> Option<&T> {
        let last_pos = self.last_idx()?;
        self.0[last_pos].as_ref()
    }

    /// Returns the index of the last item with a value or None if the LimitedVec is empty.
    #[inline]
    pub fn last_idx(&self) -> Option<usize> {
        self.len().checked_sub(1)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter::new(self)
    }

    /// Returns the next empty allocated item
    #[inline]
    fn next_mut(&mut self) -> Option<&mut Option<T>> {
        self.0.iter_mut().find(|i| i.is_none())
    }
}

impl<T, const N: usize> Index<usize> for LimitedVec<T, N> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match &self.0[index] {
            Some(e) => e,
            None => {
                let cap = self.capacity();
                panic!("Index {index} out of bounds with capacity of {cap}",)
            }
        }
    }
}

impl<T, const N: usize> From<Vec<T>> for LimitedVec<T, N> {
    fn from(values: Vec<T>) -> Self {
        if values.len() > N {
            panic!("Vec is larger than LimitedVec's capacity");
        }
        let free = N - values.len();
        let free_iter = (0..free).map(|_| None);
        let d: [Option<T>; N] = values
            .into_iter()
            .map(|i| Some(i))
            .chain(free_iter)
            .collect::<Vec<_>>()
            .try_into()
            .ok()
            .unwrap();
        LimitedVec(d)
    }
}

impl<T, const N: usize> FromIterator<T> for LimitedVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut buf = Vec::with_capacity(N);
        for i in iter.into_iter() {
            buf.push(Some(i));
            if buf.len() > N {
                panic!("Can't collect more elements into LimitedVec than capacity (N)");
            }
        }
        if buf.len() < N {
            let free_iter = (0..(N - buf.len())).map(|_| None);
            buf.extend(free_iter);
        }
        LimitedVec(buf.try_into().ok().unwrap())
    }
}

impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for LimitedVec<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(feature = "with_serde")]
impl<T, const N: usize> Serialize for LimitedVec<T, N>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        //serializer.serialize_i32(*self)
        let mut list = serializer.serialize_seq(Some(self.len()))?;
        for i in self.iter() {
            list.serialize_element(i)?;
        }
        list.end()
    }
}

#[cfg(feature = "with_serde")]
pub struct LimitedVecVisitor<T, const N: usize> {
    pd: std::marker::PhantomData<T>,
}

#[cfg(feature = "with_serde")]
impl<'de, T, const N: usize> Visitor<'de> for LimitedVecVisitor<T, N>
where
    T: Deserialize<'de>,
{
    type Value = LimitedVec<T, N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Error deserializing")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut elemens = Vec::with_capacity(N);
        while let Some(next) = seq.next_element::<T>()? {
            elemens.push(Some(next));
        }
        assert!(N >= elemens.len());
        elemens.extend((0..(N - elemens.len())).map(|_| None));
        Ok(LimitedVec(elemens.try_into().ok().unwrap()))
    }
}

#[cfg(feature = "with_serde")]
impl<'de, T, const N: usize> Deserialize<'de> for LimitedVec<T, N>
where
    T: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(LimitedVecVisitor::<T, N> {
            pd: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::LimitedVec;

    #[test]
    fn test_len() {
        const SIZE: usize = 4;
        let mut vec: LimitedVec<u8, SIZE> = LimitedVec::new();
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), SIZE);

        vec.push(42);
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.free(), SIZE - 1);

        vec.push(69);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.free(), SIZE - 2);

        assert_eq!(vec.get(0), Some(&42));
        assert_eq!(vec[0], 42);

        assert_eq!(vec.pop(), Some(69));
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.free(), SIZE - 1);

        assert_eq!(vec.pop(), Some(42));
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.free(), SIZE);
    }

    #[test]
    fn test_from_vec() {
        const SIZE: usize = 10;
        let src_vec = (0..7).collect::<Vec<usize>>();
        let lvec = LimitedVec::<_, SIZE>::from(src_vec.clone());
        assert_eq!(
            lvec.iter().collect::<Vec<_>>(),
            src_vec.iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_from_iter() {
        const SIZE: usize = 10;
        let src_vec = (0..7).collect::<Vec<usize>>();
        let lvec = (0..7).collect::<LimitedVec<usize, SIZE>>();
        assert_eq!(
            lvec.iter().collect::<Vec<_>>(),
            src_vec.iter().collect::<Vec<_>>()
        );
    }

    #[cfg(feature = "with_serde")]
    #[test]
    fn test_bincode() {
        let iter = (0..10).map(|i| format!("Number: {i}"));
        let lvec = LimitedVec::<String, 14>::from_iter(iter);

        let encoded = bincode::serialize(&lvec).unwrap();
        let decoded: LimitedVec<String, 14> = bincode::deserialize(&encoded).unwrap();

        assert_eq!(lvec, decoded);
    }
}
