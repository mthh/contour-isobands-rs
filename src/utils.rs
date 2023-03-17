use crate::area::area;
use crate::isobands::Cell;
use geo_types::Coord;

pub(crate) fn is_winding_correct(points: &[Coord<f64>], is_exterior: bool) -> bool {
    let area = area(points);
    if is_exterior {
        area > 0f64
    } else {
        area < 0f64
    }
}

pub(crate) fn empty_cell_grid(li: usize, lj: usize) -> Vec<Vec<Option<Cell>>> {
    let mut cell_grid: Vec<Vec<Option<Cell>>> = Vec::with_capacity(li - 1);
    for i in 0..li - 1 {
        cell_grid.push(Vec::with_capacity(lj - 1));
        for _j in 0..lj - 1 {
            cell_grid[i].push(None);
        }
    }
    cell_grid
}
