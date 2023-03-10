pub(crate) type GridCoord = (usize, usize);

pub(crate) trait GridTrait<T> {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn has(&self, p: &GridCoord) -> bool;
    fn get(&self, p: &GridCoord) -> Option<&T>;
}

#[derive(Debug)]
pub(crate) struct Grid<T> {
    array: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Grid<T> {
    pub fn new(width: usize, height: usize) -> Self
    where
        T: Default + Copy,
    {
        Self {
            array: [T::default()].repeat(width * height),
            width,
            height,
        }
    }

    pub fn new_from_vec(vec: Vec<T>, width: usize, height: usize) -> Self {
        // We are only using it internally
        // and we already checked that the array was of size width * height.
        Self {
            array: vec,
            width,
            height,
        }
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &[T]> {
        self.array.chunks(self.width)
    }

    /// Iterate over all elements in the grid, returning the element and its
    /// coordinates on the width and height axis.
    pub fn iter(&self) -> impl Iterator<Item = (&T, usize, usize)> {
        self.array
            .iter()
            .enumerate()
            .map(|(i, v)| (v, i % self.width, i / self.width))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut T, usize, usize)> {
        self.array
            .iter_mut()
            .enumerate()
            .map(|(i, v)| (v, i % self.width, i / self.width))
    }
}

impl<T> GridTrait<T> for Grid<T> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn has(&self, p: &GridCoord) -> bool {
        p.0 < self.width && p.1 < self.height
    }

    fn get(&self, p: &GridCoord) -> Option<&T> {
        if !self.has(p) {
            None
        } else {
            Some(unsafe { self.array.get_unchecked(p.1 * self.width + p.0) })
        }
    }
}

impl<T> std::ops::Index<GridCoord> for Grid<T> {
    type Output = T;

    fn index(&self, p: GridCoord) -> &Self::Output {
        unsafe { self.array.get_unchecked(p.1 * self.width + p.0) }
    }
}

impl<T> std::ops::IndexMut<GridCoord> for Grid<T> {
    fn index_mut(&mut self, p: GridCoord) -> &mut Self::Output {
        unsafe { self.array.get_unchecked_mut(p.1 * self.width + p.0) }
    }
}

impl<T> From<&[Vec<T>]> for Grid<T>
where
    T: Copy,
{
    fn from(vec: &[Vec<T>]) -> Self {
        let width = vec[0].len();
        let height = vec.len();
        let mut new_vec = Vec::with_capacity(width * height);
        for row in vec {
            new_vec.extend_from_slice(row);
        }
        Grid::new_from_vec(new_vec, width, height)
    }
}

pub(crate) struct BorrowedGrid<'a, T> {
    array: &'a [T],
    width: usize,
    height: usize,
}

impl<'a, T> BorrowedGrid<'a, T> {
    pub fn new(array: &'a [T], width: usize, height: usize) -> Self {
        // We are only using it internally
        // and we already checked that the array was of size width * height.
        Self {
            array,
            width,
            height,
        }
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &[T]> {
        self.array.chunks(self.width)
    }
}

impl<'a, T> GridTrait<T> for BorrowedGrid<'a, T> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn has(&self, p: &GridCoord) -> bool {
        p.0 < self.width && p.1 < self.height
    }

    fn get(&self, p: &GridCoord) -> Option<&T> {
        if !self.has(p) {
            None
        } else {
            Some(unsafe { self.array.get_unchecked(p.1 * self.width + p.0) })
        }
    }
}

impl<'a, T> std::ops::Index<GridCoord> for BorrowedGrid<'a, T> {
    type Output = T;

    fn index(&self, p: GridCoord) -> &Self::Output {
        unsafe { self.array.get_unchecked(p.1 * self.width + p.0) }
    }
}

// impl<'a, T> std::ops::IndexMut<GridCoord> for BorrowedGrid<'a, T> {
//     fn index_mut(&mut self, p: GridCoord) -> &mut Self::Output {
//         unsafe { self.array.get_unchecked_mut(p.1 * self.width + p.0) }
//     }
// }
