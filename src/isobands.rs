use std::cmp::Ordering;
use crate::area::{area, contains};
use crate::errors::{new_error, Error, ErrorKind, Result};
use crate::grid::BorrowedGrid;
use crate::polygons::trace_band_paths;
use crate::quadtree::QuadTree;
use crate::shape_coordinates::prepare_cell;
use geo_types::{LineString, MultiPolygon, Point, Polygon};
use rustc_hash::FxHashMap;

/// A point, as a tuple, where the first element is the x coordinate
/// and the second is the y coordinate.
#[derive(Debug, Clone, PartialEq)]
pub struct Pt(pub f64, pub f64);

/// The raw result of the isoband computation,
/// where the first element is a vector of paths,
/// the second is the minimum value
/// and the third is the maximum value.
pub type BandRaw = (Vec<Vec<Point<f64>>>, f64, f64);

/// An isoband, described by its min and max value and MultiPolygon.
#[derive(Debug)]
pub struct Band {
    /// The minimum value of the isoband
    pub min_v: f64,
    /// The maximum value of the isoband
    pub max_v: f64,
    /// The MultiPolygon enclosing the points between min_v and max_v
    pub geometry: MultiPolygon<f64>,
}

impl Band {
    pub fn geometry(&self) -> &MultiPolygon<f64> {
        &self.geometry
    }

    pub fn into_inner(self) -> (MultiPolygon<f64>, f64, f64) {
        (self.geometry, self.min_v, self.max_v)
    }

    pub fn min_v(&self) -> f64 {
        self.min_v
    }

    pub fn max_v(&self) -> f64 {
        self.max_v
    }

    #[cfg(feature = "geojson")]
    /// Convert the isoband to a GeoJSON Feature
    ///
    /// To get a string representation, call to_geojson().to_string().
    pub fn to_geojson(&self) -> geojson::Feature {
        let mut properties = geojson::JsonObject::with_capacity(2);
        properties.insert("min_v".to_string(), self.min_v.into());
        properties.insert("max_v".to_string(), self.max_v.into());

        geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::from(self.geometry())),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Cell {
    pub cval: usize,
    pub x0: f64,
    pub x1: f64,
    pub x2: f64,
    pub x3: f64,
    pub edges: FxHashMap<EnterType, Edge>,
    // pub x: usize,
    // pub y: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum EnterType {
    TL,
    LT,
    LB,
    BL,
    BR,
    RB,
    RT,
    TR,
}

#[derive(Debug)]
pub(crate) struct MoveInfo {
    pub x: i32,
    pub y: i32,
    pub enter: EnterType,
}

#[derive(Debug)]
pub(crate) struct Edge {
    pub path: [Pt; 2],
    pub move_info: MoveInfo,
}

#[derive(Debug)]
pub(crate) struct Settings {
    pub min_v: f64,
    pub max_v: f64,
}

static PRECISION: f64 = 1e-4;

/// Contours generator, using builder pattern, to
/// be used on a rectangular `Slice` of values to
/// get a `Vec` of [`Band`] (uses [`isobands`] function
/// internally).
///
/// [`isobands`]: fn.isobands.html
pub struct ContourBuilder {
    /// The width of the grid
    width: usize,
    /// The height of the grid
    height: usize,
    /// The horizontal coordinate for the origin of the grid.
    x_origin: f64,
    /// The vertical coordinate for the origin of the grid.
    y_origin: f64,
    /// The horizontal step for the grid
    x_step: f64,
    /// The vertical step for the grid
    y_step: f64,
    /// Whether to use a quadtree
    use_quad_tree: bool,
}

impl ContourBuilder {
    /// Constructs a new contours generator for a grid of size `width` x `height`.
    ///
    /// By default, `x_origin` and `y_origin` are set to `0.0`, `x_step` and `y_step` to `1.0`
    /// and `use_quad_tree` to `false`.
    /// This can be changed using the corresponding methods.
    pub fn new(width: usize, height: usize) -> Self {
        ContourBuilder {
            width,
            height,
            x_origin: 0f64,
            y_origin: 0f64,
            x_step: 1f64,
            y_step: 1f64,
            use_quad_tree: false,
        }
    }

    /// Sets the x origin of the grid.
    pub fn x_origin(mut self, x_origin: impl Into<f64>) -> Self {
        self.x_origin = x_origin.into();
        self
    }

    /// Sets the y origin of the grid.
    pub fn y_origin(mut self, y_origin: impl Into<f64>) -> Self {
        self.y_origin = y_origin.into();
        self
    }

    /// Sets the x step of the grid.
    pub fn x_step(mut self, x_step: impl Into<f64>) -> Self {
        self.x_step = x_step.into();
        self
    }

    /// Sets the y step of the grid.
    pub fn y_step(mut self, y_step: impl Into<f64>) -> Self {
        self.y_step = y_step.into();
        self
    }

    /// Sets whether to use a quadtree.
    pub fn use_quad_tree(mut self, use_quad_tree: bool) -> Self {
        self.use_quad_tree = use_quad_tree;
        self
    }

    /// Generates contours for the given data and thresholds.
    pub fn contours(&self, data: &[f64], thresholds: &[f64]) -> Result<Vec<Band>> {
        let mut bands = isobands(
            data,
            thresholds,
            self.use_quad_tree,
            self.width,
            self.height,
        )?;
        // Use x_origin, y_origin, x_step and y_step to calculate the coordinates of the points
        // if they are not the default values
        if (self.x_origin, self.y_origin) != (0f64, 0f64)
            || (self.x_step, self.y_step) != (1f64, 1f64)
        {
            let res = bands
                .drain(..)
                .map(|(mut raw_band, min_v, max_v)| {
                    let polygons: MultiPolygon = raw_band
                        .drain(..)
                        .map(|mut poly| {
                            poly.iter_mut().for_each(|point| {
                                let pt_x = point.x_mut();
                                *pt_x = self.x_origin + *pt_x * self.x_step;
                                let pt_y = point.y_mut();
                                *pt_y = self.y_origin + *pt_y * self.y_step;
                            });
                            Polygon::new(poly.into(), vec![])
                        })
                        .collect::<Vec<Polygon<f64>>>()
                        .into();
                    Band {
                        geometry: polygons,
                        min_v,
                        max_v,
                    }
                })
                .collect::<Vec<Band>>();
            Ok(res)
        } else {
            let res = bands
                .drain(..)
                .map(|(mut raw_band, min_v, max_v)| {
                    let polygons: MultiPolygon = raw_band
                        .drain(..)
                        .map(|points| Polygon::new(points.into(), vec![]))
                        .collect::<Vec<Polygon<f64>>>()
                        .into();
                    Band {
                        geometry: polygons.into(),
                        min_v,
                        max_v,
                    }
                })
                .collect::<Vec<Band>>();
            Ok(res)
        }
    }
}

/// Generates contours for the given data and thresholds.
/// Returns a `Vec` of [`BandRaw`] (this is the raw result
/// of the marching squares algorithm,
/// before converting the paths to geo_types geometries).
pub fn isobands(
    data: &[f64],
    thresholds: &[f64],
    use_quad_tree: bool,
    width: usize,
    height: usize,
) -> Result<Vec<BandRaw>> {
    if data.is_empty() {
        return Err(new_error(ErrorKind::BadData));
    }
    if data.len() != width * height {
        return Err(new_error(ErrorKind::BadDimension));
    }
    if thresholds.len() < 2 {
        return Err(new_error(ErrorKind::BadIntervals));
    }

    let data = BorrowedGrid::new(data, width, height);
    if use_quad_tree {
        _isobands_quadtree_raw(data, thresholds)
    } else {
        _isobands_raw(data, thresholds)
    }
}

fn empty_cell_grid(li: usize, lj: usize) -> Vec<Vec<Option<Cell>>> {
    let mut cell_grid: Vec<Vec<Option<Cell>>> = Vec::with_capacity(li - 1);
    for i in 0..li - 1 {
        cell_grid.push(Vec::with_capacity(lj - 1));
        for _j in 0..lj - 1 {
            cell_grid[i].push(None);
        }
    }
    cell_grid
}

fn _isobands_raw(data: BorrowedGrid<f64>, thresholds: &[f64]) -> Result<Vec<BandRaw>> {
    let lj = data.height();
    let li = data.width();
    let n_pair_thresholds = thresholds.len() - 1;

    // Allocate the cell grid once
    let mut cell_grid: Vec<Vec<Option<Cell>>> = empty_cell_grid(li, lj);

    let res = thresholds
        .iter()
        .zip(thresholds.iter().skip(1))
        .enumerate()
        .map(|(i, (&min, &max))| -> Result<BandRaw> {
            // Store min / max values for the current band
            let opt = Settings {
                min_v: min,
                max_v: if i + 1 == n_pair_thresholds {
                    max
                } else {
                    max - PRECISION
                },
            };

            // Fill up the grid with cell information
            cell_grid.iter_mut().enumerate().try_for_each(|(i, row)| {
                row.iter_mut().enumerate().try_for_each(|(j, cell)| {
                    *cell = prepare_cell(i, j, &data, &opt)?;
                    Ok::<(), Error>(())
                })
            })?;

            let band_polygons = trace_band_paths(&data, &mut cell_grid, &opt)?;
            Ok((band_polygons, min, max))
        })
        .collect::<Result<Vec<BandRaw>>>()?;

    Ok(res)
}

fn _isobands_quadtree_raw(data: BorrowedGrid<f64>, thresholds: &[f64]) -> Result<Vec<BandRaw>> {
    let lj = data.height();
    let li = data.width();
    let n_pair_thresholds = thresholds.len() - 1;

    // Instantiate the quadtree
    let tree = QuadTree::new(&data);

    // Allocate the cell grid once
    let mut cell_grid: Vec<Vec<Option<Cell>>> = empty_cell_grid(li, lj);

    let res = thresholds
        .iter()
        .zip(thresholds.iter().skip(1))
        .enumerate()
        .map(|(i, (&min, &max))| -> Result<BandRaw> {
            // Store min / max values for the current band
            let opt = Settings {
                min_v: min,
                max_v: if i + 1 == n_pair_thresholds {
                    max
                } else {
                    max - PRECISION
                },
            };

            // Clear the grid
            if i > 0 {
                cell_grid.iter_mut().for_each(|row| {
                    row.iter_mut().for_each(|cell| {
                        *cell = None;
                    })
                });
            }

            // Fill up the grid with cell information
            for (i, j) in tree.cells_in_band(opt.min_v, opt.max_v) {
                cell_grid[i][j] = prepare_cell(i, j, &data, &opt)?;
            }

            let band_polygons = trace_band_paths(&data, &mut cell_grid, &opt)?;

            Ok((band_polygons, min, max))
        })
        .collect::<Result<Vec<BandRaw>>>()?;

    Ok(res)
}
