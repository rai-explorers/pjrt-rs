pub trait Shape {
    fn shape(&self) -> &[i64];

    #[inline]
    fn rank(&self) -> usize {
        self.shape().len()
    }
}

impl<'a, T> Shape for &'a T
where
    T: Shape + ?Sized,
{
    fn shape(&self) -> &[i64] {
        (*self).shape()
    }
}

impl Shape for Vec<i64> {
    fn shape(&self) -> &[i64] {
        self.as_slice()
    }

    fn rank(&self) -> usize {
        self.len()
    }
}

impl Shape for [i64] {
    fn shape(&self) -> &[i64] {
        self
    }

    fn rank(&self) -> usize {
        self.len()
    }
}

impl<const N: usize> Shape for [i64; N] {
    fn shape(&self) -> &[i64] {
        self.as_slice()
    }

    fn rank(&self) -> usize {
        self.len()
    }
}
