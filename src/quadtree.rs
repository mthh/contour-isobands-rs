use crate::grid::{BorrowedGrid, GridCoord, GridTrait};

#[derive(Debug)]
pub(crate) struct TreeNode {
    child_a: Option<Box<TreeNode>>,
    child_b: Option<Box<TreeNode>>,
    child_c: Option<Box<TreeNode>>,
    child_d: Option<Box<TreeNode>>,
    x: usize,
    y: usize,
    lower_bound: f64,
    upper_bound: f64,
}

impl TreeNode {
    pub fn new(data: &BorrowedGrid<f64>, x: usize, y: usize, dx: usize, dy: usize) -> TreeNode {
        let mut dx_tmp = dx;
        let mut dy_tmp = dy;
        let mut msb_x = 0;
        let mut msb_y = 0;

        let mut tn = TreeNode {
            child_a: None,
            child_b: None,
            child_c: None,
            child_d: None,
            x,
            y,
            lower_bound: 0.0,
            upper_bound: 0.0,
        };

        if dx == 1 && dy == 1 {
            // tn.lower_bound =
            //     data[(y, x)].min(data[(y + 1, x)].min(data[(y, x + 1)].min(data[(y + 1, x + 1)])));
            // tn.upper_bound =
            //     data[(y, x)].max(data[(y + 1, x)].max(data[(y, x + 1)].max(data[(y + 1, x + 1)])));
            tn.lower_bound =
                data[(x, y)].min(data[(x + 1, y)].min(data[(x, y + 1)].min(data[(x + 1, y + 1)])));
            tn.upper_bound =
                data[(x, y)].max(data[(x + 1, y)].max(data[(x, y + 1)].max(data[(x + 1, y + 1)])));
        } else {
            // Get most significant bit from dx
            if dx > 1 {
                while dx_tmp != 0 {
                    dx_tmp >>= 1;
                    msb_x += 1;
                }
                if dx == (1 << (msb_x - 1)) {
                    msb_x -= 1;
                }

                dx_tmp = 1 << (msb_x - 1);
            }

            if dy > 1 {
                while dy_tmp != 0 {
                    dy_tmp >>= 1;
                    msb_y += 1;
                }
                if dy == (1 << (msb_y - 1)) {
                    msb_y -= 1;
                }

                dy_tmp = 1 << (msb_y - 1);
            }

            tn.child_a = Some(Box::new(TreeNode::new(data, x, y, dx_tmp, dy_tmp)));
            tn.lower_bound = tn.child_a.as_ref().unwrap().lower_bound;
            tn.upper_bound = tn.child_a.as_ref().unwrap().upper_bound;

            if dx - dx_tmp > 0 {
                tn.child_b = Some(Box::new(TreeNode::new(
                    data,
                    x + dx_tmp,
                    y,
                    dx - dx_tmp,
                    dy_tmp,
                )));
                tn.lower_bound = tn.lower_bound.min(tn.child_b.as_ref().unwrap().lower_bound);
                tn.upper_bound = tn.upper_bound.max(tn.child_b.as_ref().unwrap().upper_bound);

                if dy - dy_tmp > 0 {
                    tn.child_c = Some(Box::new(TreeNode::new(
                        data,
                        x + dx_tmp,
                        y + dy_tmp,
                        dx - dx_tmp,
                        dy - dy_tmp,
                    )));
                    tn.lower_bound = tn.lower_bound.min(tn.child_c.as_ref().unwrap().lower_bound);
                    tn.upper_bound = tn.upper_bound.max(tn.child_c.as_ref().unwrap().upper_bound);
                }
            }

            if dy - dy_tmp > 0 {
                tn.child_d = Some(Box::new(TreeNode::new(
                    data,
                    x,
                    y + dy_tmp,
                    dx_tmp,
                    dy - dy_tmp,
                )));
                tn.lower_bound = tn.lower_bound.min(tn.child_d.as_ref().unwrap().lower_bound);
                tn.upper_bound = tn.upper_bound.max(tn.child_d.as_ref().unwrap().upper_bound);
            }
        }

        tn
    }

    pub fn cells_in_band(&self, lowerbound: f64, upperbound: f64) -> Vec<GridCoord> {
        let mut cells = Vec::new();

        if self.lower_bound > upperbound || self.upper_bound < lowerbound {
            cells
        } else if !(self.child_a.is_some()
            || self.child_b.is_some()
            || self.child_c.is_some()
            || self.child_d.is_some())
        {
            cells.push((self.x, self.y));
            cells
        } else {
            if self.child_a.is_some() {
                cells.extend(
                    self.child_a
                        .as_ref()
                        .unwrap()
                        .cells_in_band(lowerbound, upperbound),
                );
            }
            if self.child_b.is_some() {
                cells.extend(
                    self.child_b
                        .as_ref()
                        .unwrap()
                        .cells_in_band(lowerbound, upperbound),
                );
            }
            if self.child_c.is_some() {
                cells.extend(
                    self.child_c
                        .as_ref()
                        .unwrap()
                        .cells_in_band(lowerbound, upperbound),
                );
            }
            if self.child_d.is_some() {
                cells.extend(
                    self.child_d
                        .as_ref()
                        .unwrap()
                        .cells_in_band(lowerbound, upperbound),
                );
            }
            cells
        }
    }
}

#[derive(Debug)]
pub(crate) struct QuadTree {
    root: TreeNode,
}

impl QuadTree {
    pub fn new(data: &BorrowedGrid<f64>) -> QuadTree {
        QuadTree {
            root: TreeNode::new(data, 0, 0, data.width() - 1, data.height() - 1),
        }
    }

    pub fn cells_in_band(&self, lowerbound: f64, upperbound: f64) -> Vec<GridCoord> {
        self.root.cells_in_band(lowerbound, upperbound)
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::BorrowedGrid;
    use crate::quadtree::QuadTree;

    #[test]
    fn test_partition_quadtree() {
        let data = vec![
            1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12., 13., 14., 15., 16.,
        ];
        let width = 4;
        let height = 4;
        let grid = BorrowedGrid::new(&data, width, height);
        let quadtree = QuadTree::new(&grid);

        assert_eq!(quadtree.root.child_a.as_ref().unwrap().lower_bound, 1.);
        assert_eq!(quadtree.root.child_a.as_ref().unwrap().upper_bound, 11.);
        assert_eq!(quadtree.root.child_a.as_ref().unwrap().x, 0);
        assert_eq!(quadtree.root.child_a.as_ref().unwrap().y, 0);
        assert_eq!(quadtree.root.child_b.as_ref().unwrap().lower_bound, 3.);
        assert_eq!(quadtree.root.child_b.as_ref().unwrap().upper_bound, 12.);
        assert_eq!(quadtree.root.child_b.as_ref().unwrap().x, 2);
        assert_eq!(quadtree.root.child_b.as_ref().unwrap().y, 0);
        assert_eq!(quadtree.root.child_c.as_ref().unwrap().lower_bound, 11.);
        assert_eq!(quadtree.root.child_c.as_ref().unwrap().upper_bound, 16.);
        assert_eq!(quadtree.root.child_c.as_ref().unwrap().x, 2);
        assert_eq!(quadtree.root.child_c.as_ref().unwrap().y, 2);
        assert_eq!(quadtree.root.child_d.as_ref().unwrap().lower_bound, 9.);
        assert_eq!(quadtree.root.child_d.as_ref().unwrap().upper_bound, 15.);
        assert_eq!(quadtree.root.child_d.as_ref().unwrap().x, 0);
        assert_eq!(quadtree.root.child_d.as_ref().unwrap().y, 2);
    }
}
