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
        if vec.len() != width * height {
            panic!("Invalid grid dimensions");
        }
        Self {
            array: vec,
            width,
            height,
        }
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &[T]> {
        self.array.chunks(self.width)
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
        // if !self.has(&p) {
        //     panic!("Invalid grid coordinate");
        // }
        unsafe { self.array.get_unchecked(p.1 * self.width + p.0) }
    }
}

impl<T> std::ops::IndexMut<GridCoord> for Grid<T> {
    fn index_mut(&mut self, p: GridCoord) -> &mut Self::Output {
        // if !self.has(&p) {
        //     panic!("Invalid grid coordinate");
        // }
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
        if array.len() != width * height {
            panic!("Invalid grid dimensions");
        }
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
        // if !self.has(&p) {
        //     panic!("Invalid grid coord");
        // }
        unsafe { self.array.get_unchecked(p.1 * self.width + p.0) }
    }
}

// impl<'a, T> std::ops::IndexMut<GridCoord> for BorrowedGrid<'a, T> {
//     fn index_mut(&mut self, p: GridCoord) -> &mut Self::Output {
//         // if !self.has(&p) {
//         //     panic!("Invalid grid coord");
//         // }
//         unsafe { self.array.get_unchecked_mut(p.1 * self.width + p.0) }
//     }
// }
