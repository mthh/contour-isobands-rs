use crate::errors::{ErrorKind, new_error, Result};
use crate::polygons::trace_band_paths;
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

pub fn isobands(data: &[Vec<f64>], intervals: &[f64]) -> Result<Vec<Band>> {
    if intervals.len() < 2 {
        return Err(new_error(ErrorKind::BadIntervals));
    }
    let lj = data.len();
    let li = data[0].len();

    let res = intervals
        .iter()
        .zip(intervals.iter().skip(1))
        .map(|(min, max)| -> Result<Band> {
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
            let band_polygons = trace_band_paths(data, &mut cell_grid, &opt)?;

            let polygons: MultiPolygon<f64> = band_polygons
                .iter()
                .map(|poly| {
                    let points: Vec<Point<f64>> = poly
                        .iter()
                        .map(|pt| Point::new(pt.0, pt.1))
                        .collect::<Vec<Point<f64>>>();
                    Polygon::new(points.into(), vec![])
                })
                .collect::<Vec<Polygon<f64>>>()
                .into();
            Ok(Band {
                polygons,
                min_v: opt.min_v,
                max_v: opt.max_v,
            })
        })
        .collect::<Result<Vec<Band>>>()?;

    Ok(res)
}

pub fn isobands_test(data: &[Vec<f64>], intervals: &[f64]) -> Result<Vec<Vec<Vec<Pt>>>> {
    if intervals.len() < 2 {
        return Err(new_error(ErrorKind::BadIntervals));
    }
    let lj = data.len();
    let li = data[0].len();

    let res = intervals
          .iter()
          .zip(intervals.iter().skip(1))
          .map(|(min, max)| -> Result<Vec<Vec<Pt>>> {
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
            Ok(band_polygons)
        })
        .collect::<Result<Vec<Vec<Vec<Pt>>>>>()?;

    Ok(res)
}
