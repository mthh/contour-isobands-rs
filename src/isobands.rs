use crate::errors::{new_error, Error, ErrorKind, Result};
use crate::polygons::trace_band_paths;
use crate::shape_coordinates::prepare_cell;
use geo_types::{Coord, MultiPolygon, Point, Polygon};
// use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Pt(pub f64, pub f64);

impl From<Pt> for Coord<f64> {
    fn from(pt: Pt) -> Self {
        Coord { x: pt.0, y: pt.1 }
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

// impl FromStr for EnterType {
//     type Err = Error;
//
//     fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
//         match s {
//             "TL" => Ok(EnterType::TL),
//             "LT" => Ok(EnterType::LT),
//             "LB" => Ok(EnterType::LB),
//             "BL" => Ok(EnterType::BL),
//             "BR" => Ok(EnterType::BR),
//             "RB" => Ok(EnterType::RB),
//             "RT" => Ok(EnterType::RT),
//             "TR" => Ok(EnterType::TR),
//             _ => Err(new_error(ErrorKind::UnknownEnterType)),
//         }
//     }
// }

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

pub fn isobands(data: &[Vec<f64>], min_v: &[f64], bandwidth: &[f64]) -> Result<Vec<Vec<Vec<Pt>>>> {
    if min_v.len() != bandwidth.len() {
        panic!("min_v and bandwidth must have the same length");
    }
    let lj = data.len();
    let li = data[0].len();

    let res = min_v
        .iter()
        .zip(bandwidth.iter())
        .map(|(min, bw)| -> Result<Vec<Vec<Pt>>> {
            let opt = Settings {
                min_v: *min,
                max_v: *min + *bw,
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