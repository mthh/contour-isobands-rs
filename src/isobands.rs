use crate::area::{area, contains};
use crate::errors::{new_error, ErrorKind, Result};
use crate::polygons::trace_band_paths;
use crate::quadtree::QuadTree;
use crate::shape_coordinates::prepare_cell;
use geo_types::{Coord, MultiPolygon, Point, Polygon};

#[derive(Debug, Clone, PartialEq)]
pub struct Pt(pub f64, pub f64);

impl From<Pt> for Coord<f64> {
    fn from(pt: Pt) -> Self {
        Coord { x: pt.0, y: pt.1 }
    }
}

/// An isoband, described by its min and max value and MultiPolygon.
#[derive(Debug)]
pub struct Band {
    pub min_v: f64,
    pub max_v: f64,
    pub polygons: MultiPolygon<f64>,
}

type BandRaw = (Vec<Vec<Pt>>, f64, f64);

impl Band {
    pub fn geometry(&self) -> &MultiPolygon<f64> {
        &self.polygons
    }

    pub fn into_inner(self) -> (MultiPolygon<f64>, f64, f64) {
        (self.polygons, self.min_v, self.max_v)
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
    pub edges: Edges,
    // pub polygons: Vec<Vec<Pt>>,
    // pub x: usize,
    // pub y: usize,
}

#[derive(Debug, Clone)]
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
pub(crate) struct Corner {
    pub path: Vec<Pt>,
    pub move_info: MoveInfo,
}

#[derive(Debug)]
pub(crate) struct Edges {
    pub lb: Option<Corner>,
    pub bl: Option<Corner>,
    pub br: Option<Corner>,
    pub rb: Option<Corner>,
    pub rt: Option<Corner>,
    pub tr: Option<Corner>,
    pub tl: Option<Corner>,
    pub lt: Option<Corner>,
}

impl Edges {
    pub fn get(&self, enter: &EnterType) -> Option<&Corner> {
        match enter {
            EnterType::TL => self.tl.as_ref(),
            EnterType::LT => self.lt.as_ref(),
            EnterType::LB => self.lb.as_ref(),
            EnterType::BL => self.bl.as_ref(),
            EnterType::BR => self.br.as_ref(),
            EnterType::RB => self.rb.as_ref(),
            EnterType::RT => self.rt.as_ref(),
            EnterType::TR => self.tr.as_ref(),
        }
    }

    pub fn get_remove(&mut self, enter: &EnterType) -> Option<Corner> {
        match enter {
            EnterType::TL => self.tl.take(),
            EnterType::LT => self.lt.take(),
            EnterType::LB => self.lb.take(),
            EnterType::BL => self.bl.take(),
            EnterType::BR => self.br.take(),
            EnterType::RB => self.rb.take(),
            EnterType::RT => self.rt.take(),
            EnterType::TR => self.tr.take(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Settings {
    pub min_v: f64,
    pub max_v: f64,
}

pub struct ContourBuilder {
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

impl Default for ContourBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContourBuilder {
    /// Constructs a new contours generator.
    ///
    /// By default, `x_origin` and `y_origin` are set to `0.0`, `x_step` and `y_step` to `1.0`
    /// and `use_quad_tree` to `false`.
    pub fn new() -> Self {
        ContourBuilder {
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
    pub fn contours(&self, data: &[Vec<f64>], thresholds: &[f64]) -> Result<Vec<Band>> {
        let bands = isobands(data, thresholds, self.use_quad_tree)?;
        // Use x_origin, y_origin, x_step and y_step to calculate the coordinates of the points
        if (self.x_origin, self.y_origin) != (0f64, 0f64)
            || (self.x_step, self.y_step) != (1f64, 1f64)
        {
            let res = bands
                .iter()
                .map(|(raw_band, min_v, max_v)| {
                    let polygons = raw_band
                        .iter()
                        .map(|poly| {
                            let points: Vec<Point<f64>> = poly
                                .iter()
                                .map(|pt| {
                                    Point::new(
                                        self.x_origin + pt.0 * self.x_step,
                                        self.y_origin + pt.1 * self.y_step,
                                    )
                                })
                                .collect::<Vec<Point<f64>>>();
                            Polygon::new(points.into(), vec![])
                        })
                        .collect::<Vec<Polygon<f64>>>()
                        .into();
                    Band {
                        polygons,
                        min_v: *min_v,
                        max_v: *max_v,
                    }
                })
                .collect::<Vec<Band>>();
            Ok(res)
        } else {
            let res = bands
                .iter()
                .map(|(raw_band, min_v, max_v)| {
                    let polygons = raw_band
                        .iter()
                        .map(|poly| {
                            println!("{:?}", area(poly));
                            let points: Vec<Point<f64>> = poly
                                .iter()
                                .map(|pt| Point::new(pt.0, pt.1))
                                .collect::<Vec<Point<f64>>>();
                            Polygon::new(points.into(), vec![])
                        })
                        .collect::<Vec<Polygon<f64>>>()
                        .into();
                    Band {
                        polygons,
                        min_v: *min_v,
                        max_v: *max_v,
                    }
                })
                .collect::<Vec<Band>>();
            Ok(res)
        }
    }
}

pub fn isobands(
    data: &[Vec<f64>],
    thresholds: &[f64],
    use_quad_tree: bool,
) -> Result<Vec<BandRaw>> {
    if data.is_empty() {
        return Err(new_error(ErrorKind::BadData));
    }
    data.iter()
        .map(|row| {
            if row.is_empty() || row.len() != data[0].len() {
                Err(new_error(ErrorKind::BadRowLength))?
            }
            Ok(())
        })
        .collect::<Result<Vec<()>>>()?;
    if thresholds.len() < 2 {
        return Err(new_error(ErrorKind::BadIntervals));
    }
    if use_quad_tree {
        _isobands_quadtree_raw(data, thresholds)
    } else {
        _isobands_raw(data, thresholds)
    }
}

pub fn _isobands_raw(data: &[Vec<f64>], thresholds: &[f64]) -> Result<Vec<BandRaw>> {
    let lj = data.len();
    let li = data[0].len();

    let res = thresholds
        .iter()
        .zip(thresholds.iter().skip(1))
        .map(|(min, max)| -> Result<BandRaw> {
            let opt = Settings {
                min_v: *min,
                max_v: *max,
            };

            let mut cell_grid = Vec::with_capacity(li);
            for i in 0..li - 1 {
                cell_grid.push(Vec::with_capacity(lj));
                for j in 0..lj - 1 {
                    cell_grid[i].push(prepare_cell(i, j, data, &opt)?);
                }
            }
            // println!("cell_grid: {:?}", cell_grid);
            let band_polygons = trace_band_paths(data, &mut cell_grid, &opt)?;
            // Todo: remove this when done debugging
            display_debug_info(&band_polygons);
            Ok((band_polygons, opt.min_v, opt.max_v))
        })
        .collect::<Result<Vec<BandRaw>>>()?;

    Ok(res)
}

pub fn _isobands_quadtree_raw(data: &[Vec<f64>], thresholds: &[f64]) -> Result<Vec<BandRaw>> {
    let lj = data.len();
    let li = data[0].len();

    let tree = QuadTree::new(data);
    let res = thresholds
        .iter()
        .zip(thresholds.iter().skip(1))
        .map(|(min, max)| -> Result<BandRaw> {
            let opt = Settings {
                min_v: *min,
                max_v: *max,
            };

            let mut cell_grid = Vec::with_capacity(li);
            for i in 0..li - 1 {
                cell_grid.push(Vec::with_capacity(lj));
                for _j in 0..lj - 1 {
                    cell_grid[i].push(None);
                }
            }

            for cell in tree.cells_in_band(opt.min_v, opt.max_v) {
                let i = cell.0;
                let j = cell.1;
                cell_grid[i][j] = prepare_cell(i, j, data, &opt)?;
            }
            // println!("cell_grid: {:?}", cell_grid);
            let band_polygons = trace_band_paths(data, &mut cell_grid, &opt)?;
            // Todo: remove this when done debugging
            display_debug_info(&band_polygons);
            Ok((band_polygons, opt.min_v, opt.max_v))
        })
        .collect::<Result<Vec<BandRaw>>>()?;

    Ok(res)
}

fn display_debug_info(band_polygons: &[Vec<Pt>]) {
    println!("Start band");
    let n_poly = band_polygons.len();
    println!("n_poly = {:?}", n_poly);
    for b in band_polygons {
        println!("area = {:?}", area(b));
        if n_poly > 1 {
            println!(
                "last poly contains this one : {:?}",
                contains(&band_polygons[n_poly - 1], b)
            );
            println!(
                "first poly contains this one : {:?}",
                contains(&band_polygons[0], b)
            );
        }
    }
    println!("End band\n");
}
