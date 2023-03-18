use crate::contains::contains;
use crate::errors::{new_error, Error, ErrorKind, Result};
use crate::grid::BorrowedGrid;
use crate::polygons::trace_band_paths;
use crate::quadtree::QuadTree;
use crate::shape_coordinates::prepare_cell;
use crate::utils::{empty_cell_grid, is_winding_correct};
use geo_types::{LineString, MultiPolygon, Point, Polygon};
use rustc_hash::FxHashMap;

use crate::area::area;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// A point, as a tuple, where the first element is the x coordinate
/// and the second is the y coordinate.
#[derive(Debug, Clone, PartialEq)]
pub struct Pt(pub f64, pub f64);

/// The raw result of the isoband computation,
/// where the first element is a vector of paths,
/// the second is the minimum value
/// and the third is the maximum value.
/// See the [`Band`] struct for a more convenient representation.
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
    pub x0: f64,
    pub x1: f64,
    pub x2: f64,
    pub x3: f64,
    pub edges: FxHashMap<EnterType, Edge>,
    // pub cval: u8,
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
    /// Winding order
    ensure_rings_orientation: bool,
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
            ensure_rings_orientation: true,
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

    /// Sets whether to ensure the winding order of the rings.
    pub fn ensure_rings_orientation(mut self, ensure_rings_orientation: bool) -> Self {
        self.ensure_rings_orientation = ensure_rings_orientation;
        self
    }

    /// Generates contour MultiPolygons for the given data and thresholds.
    pub fn contours(&self, data: &[f64], thresholds: &[f64]) -> Result<Vec<Band>> {
        // Generate the paths for each threshold (returned as a Vec of BandRaw)
        let bands = isobands(
            data,
            thresholds,
            self.use_quad_tree,
            self.width,
            self.height,
            false,
        )?;

        // Build a MultiPolygon for each band
        // and returns a Vec of Band
        let res = bands
            .into_iter()
            .map(|(raw_band, min_v, max_v)| {
                self.convert_rings_to_multipolygon(raw_band, min_v, max_v)
            })
            .collect::<Result<Vec<Band>>>()?;

        Ok(res)
    }

    /// Generates contour MultiPolygons for the given data and thresholds.
    #[cfg(feature = "parallel")]
    pub fn par_contours(&self, data: &[f64], thresholds: &[f64]) -> Result<Vec<Band>> {
        // Generate the paths for each threshold (returned as a Vec of BandRaw)
        let bands = isobands(
            data,
            thresholds,
            self.use_quad_tree,
            self.width,
            self.height,
            true,
        )?;

        // Build a MultiPolygon for each band
        // and returns a Vec of Band
        let res = bands
            .into_par_iter()
            .map(|(raw_band, min_v, max_v)| {
                self.convert_rings_to_multipolygon(raw_band, min_v, max_v)
            })
            .collect::<Result<Vec<Band>>>()?;

        Ok(res)
    }

    fn convert_rings_to_multipolygon(
        &self,
        raw_band: Vec<Vec<Point<f64>>>,
        min_v: f64,
        max_v: f64,
    ) -> Result<Band> {
        // First, convert the isobands paths to LineStrings
        let mut rings: Vec<(LineString<f64>, f64)> = raw_band
            .into_iter()
            .map(|mut points| {
                if (self.x_origin, self.y_origin) != (0f64, 0f64)
                    || (self.x_step, self.y_step) != (1f64, 1f64)
                {
                    // Use x_origin, y_origin, x_step and y_step to calculate the coordinates of the points
                    // if they are not the default values
                    points.iter_mut().for_each(|point| {
                        let pt_x = point.x_mut();
                        *pt_x = self.x_origin + *pt_x * self.x_step;
                        let pt_y = point.y_mut();
                        *pt_y = self.y_origin + *pt_y * self.y_step;
                    });
                }
                // Sometimes paths have repeated points, so we remove them
                points.dedup();
                points
            })
            // We dont want 'empty' rings
            .filter(|potential_ring| potential_ring.len() > 2)
            // We compute the area now as we will need it to sort the rings
            // (+ also later to check if a ring is clockwise or not)
            .map(|points| {
                let closed_linestring: LineString = points.into();
                let area = area(&closed_linestring.0);
                (closed_linestring, area)
            })
            .collect::<Vec<(LineString<f64>, f64)>>();

        // We sort by absolute area, so that the smallest rings are first
        // (this will help later when we reconstruct the polygons by checking which rings are enclosed by others
        // in for rings enclosed by more than one other ring, we will keep the smallest one)
        rings.sort_by_key(|(_, area)| area.abs() as u64);

        // Then we compute how many times a ring is enclosed by another ring
        let mut enclosed_by_n = FxHashMap::default();

        for (i, (ring, _)) in rings.iter().enumerate() {
            let mut enclosed_by_j = 0;
            for (j, (ring_test, _)) in rings.iter().enumerate() {
                if i == j {
                    continue;
                }
                if contains(&ring_test.0, &ring.0) {
                    enclosed_by_j += 1;
                }
            }
            enclosed_by_n.insert(i, enclosed_by_j);
        }

        // We now need to reconstruct the polygons from the rings
        let mut polygons: Vec<Polygon<f64>> = Vec::new();
        let mut interior_rings: Vec<LineString<f64>> = Vec::new();

        // First we separate the exterior rings from the interior rings
        for (i, (mut ring, ring_area)) in rings.into_iter().enumerate() {
            let enclosed_by_i = enclosed_by_n.get(&i).unwrap();
            // Rings that are enclosed by 0 other ring are Polygon exterior rings.
            // Rings that are enclosed by 1 other ring are Polygon interior rings (holes).
            // Rings that are enclosed by 2 other rings are (new) Polygon exterior rings.
            // And so on...
            if *enclosed_by_i % 2 == 0 {
                // This is an exterior ring
                // We want it to be counter-clockwise
                if self.ensure_rings_orientation && !is_winding_correct(ring_area, true) {
                    ring.0.reverse();
                }
                polygons.push(Polygon::new(ring, vec![]));
            } else {
                // This is an interior ring
                // We want it to be clockwise
                if self.ensure_rings_orientation && !is_winding_correct(ring_area, false) {
                    ring.0.reverse();
                }
                interior_rings.push(ring);
            }
        }

        // Then, for each interior ring, we find the exterior ring that encloses it
        // and add it to the polygon.
        // Due to sorting by area sooner, we should push the interior ring to the appropriate polygon
        // (in case of polygon with hole contained in another polygon with hole).
        for interior_ring in interior_rings.into_iter() {
            let mut found = false;
            for polygon in polygons.iter_mut() {
                if contains(&polygon.exterior().0, &interior_ring.0) {
                    polygon.interiors_push(interior_ring);
                    found = true;
                    break;
                }
            }
            if !found {
                // This should never happen...
                return Err(new_error(ErrorKind::PolygonReconstructionError));
            }
        }

        // Finally, we reverse the polygons so that they are in the right order
        // (this is because we sorted by area earlier,
        //  and otherwise some geos validity check can fail if Polygon 0
        //  of a MultiPolygon is inside the hole of the Polygon 1)
        polygons.reverse();

        Ok(Band {
            geometry: polygons.into(),
            min_v,
            max_v,
        })
    }
}

/// Generates contours for the given data and thresholds.
/// Returns a `Vec` of [`BandRaw`] (this is the raw result of the marching
/// squares algorithm that contains the paths of the Band as a Vec of Vec
/// of Points - this is the intermediate result that is used to build
/// the MultiPolygons in the [`ContourBuilder::contours`] method).
pub fn isobands(
    data: &[f64],
    thresholds: &[f64],
    use_quad_tree: bool,
    width: usize,
    height: usize,
    _parallel: bool,
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

    #[cfg(feature = "parallel")]
    if _parallel {
        if use_quad_tree {
            _isobands_parallel_quadtree_raw(data, thresholds)
        } else {
            _isobands_parallel_raw(data, thresholds)
        }
    } else {
        if use_quad_tree {
            _isobands_quadtree_raw(data, thresholds)
        } else {
            _isobands_raw(data, thresholds)
        }
    }
    #[cfg(not(feature = "parallel"))]
    if use_quad_tree {
        _isobands_quadtree_raw(data, thresholds)
    } else {
        _isobands_raw(data, thresholds)
    }
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

#[cfg(feature = "parallel")]
fn _isobands_parallel_raw(data: BorrowedGrid<f64>, thresholds: &[f64]) -> Result<Vec<BandRaw>> {
    let lj = data.height();
    let li = data.width();
    let n_pair_thresholds = thresholds.len() - 1;

    let res = thresholds
        .iter()
        .zip(thresholds.iter().skip(1))
        .enumerate()
        .collect::<Vec<_>>()
        .into_par_iter()
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

            let mut cell_grid: Vec<Vec<Option<Cell>>> = empty_cell_grid(li, lj);

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

#[cfg(feature = "parallel")]
fn _isobands_parallel_quadtree_raw(
    data: BorrowedGrid<f64>,
    thresholds: &[f64],
) -> Result<Vec<BandRaw>> {
    let lj = data.height();
    let li = data.width();
    let n_pair_thresholds = thresholds.len() - 1;

    // Instantiate the quadtree
    let tree = QuadTree::new(&data);

    let res = thresholds
        .iter()
        .zip(thresholds.iter().skip(1))
        .enumerate()
        .collect::<Vec<_>>()
        .into_par_iter()
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

            let mut cell_grid: Vec<Vec<Option<Cell>>> = empty_cell_grid(li, lj);

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
