#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! Compute isobands *(i.e. contour polygons which enclose
//! all the points of a grid included between two chosen values)*
//! by applying marching squares to an array of values.
//!
//! Output is a Vec of [`Band`], each [`Band`] is characterized by its minimum value,
//! its maximum value, and a [`MultiPolygon`] geometry.
//! #### Example:
#![cfg_attr(feature = "geojson", doc = "```")]
#![cfg_attr(not(feature = "geojson"), doc = "```ignore")]
//! # use contour_isobands::ContourBuilder;
//! let c = ContourBuilder::new(10, 10); // x dim., y dim.
//! let res = c.contours(&vec![
//!     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
//!     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
//!     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
//!     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
//!     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
//!     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
//!     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
//!     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
//!     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
//!     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
//! ], &[0.5, 1.]).unwrap(); // values, thresholds
//!
//! let output = serde_json::json!({
//!   "type": "Feature",
//!   "geometry": {
//!     "type": "MultiPolygon",
//!     "coordinates": [[[
//!         [3.0, 2.5], [2.5, 3.0], [2.5, 4.0], [2.5, 5.0],
//!         [2.5, 6.0], [2.5, 7.0], [3.0, 7.5], [4.0, 7.5],
//!         [5.0, 7.5], [5.5, 7.0], [5.5, 6.0], [5.5, 5.0],
//!         [5.5, 4.0], [5.5, 3.0], [5.0, 2.5], [4.0, 2.5],
//!         [3.0, 2.5]
//!     ]]]
//!   },
//!   "properties": {"min_v": 0.5, "max_v": 1.0}
//! });
//!
//! assert_eq!(res[0].to_geojson(), std::convert::TryFrom::try_from(output).unwrap());
//! ```
//! [`MultiPolygon`]: ../geo_types/geometry/struct.MultiPolygon.html
#![cfg_attr(debug_assertions, allow(dead_code))]
mod area;
mod errors;
mod grid;
mod isobands;
mod polygons;
mod quadtree;
mod shape_coordinates;

pub use crate::isobands::{isobands, Band, BandRaw, ContourBuilder};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isobands::Pt;

    fn make_grid_from2d_vec(data: &[Vec<f64>]) -> (Vec<f64>, usize, usize) {
        let width = data[0].len();
        let height = data.len();
        let mut grid: Vec<f64> = Vec::with_capacity(width * height);
        for row in data {
            grid.extend_from_slice(row);
        }
        (grid, width, height)
    }

    #[test]
    fn isobands_err_matrix_empty() {
        let matrix: Vec<Vec<f64>> = vec![vec![]];
        let (matrix, width, height) = make_grid_from2d_vec(&matrix);
        let lower_band = 1.;
        let bandwidth = 2.;

        let res = isobands(
            &matrix,
            &vec![lower_band, lower_band + bandwidth],
            false,
            width,
            height,
        );
        assert!(res.is_err());
    }

    #[test]
    fn isobands_err_threshold_too_short() {
        let matrix = vec![vec![1., 1.], vec![1., 5.]];
        let (matrix, width, height) = make_grid_from2d_vec(&matrix);

        let res = isobands(&matrix, &vec![2.], false, width, height);
        assert!(res.is_err());
    }

    #[test]
    fn isobands_err_matrix_rows_not_same_length() {
        let matrix: Vec<Vec<f64>> = vec![vec![1., 1.], vec![1., 5., 5.]];
        let (matrix, width, height) = make_grid_from2d_vec(&matrix);

        let res = isobands(&matrix, &vec![1., 3.], false, width, height);
        assert!(res.is_err());
    }

    #[test]
    fn isobands_minimal() {
        let matrix = vec![vec![1., 1.], vec![1., 5.]];
        let (matrix, width, height) = make_grid_from2d_vec(&matrix);

        let lower_band = 1.;
        let bandwidth = 2.;

        let res = isobands(
            &matrix,
            &vec![lower_band, lower_band + bandwidth],
            false,
            width,
            height,
        )
        .unwrap();
        assert_eq!(
            res[0].0,
            vec![vec![
                Pt(0.5, 1.),
                Pt(1., 0.5),
                Pt(1., 0.),
                Pt(0., 0.),
                Pt(0., 1.),
                Pt(0.5, 1.),
            ]]
        );
        assert_eq!(res[0].1, lower_band);
        assert_eq!(res[0].2, lower_band + bandwidth);
    }

    #[test]
    fn isoband_simple() {
        let matrix = vec![
            vec![1., 1., 1., 0.],
            vec![1., 5., 5., 1.],
            vec![0., 1., 1., 1.],
        ];
        let lower_band = 1.;
        let bandwidth = 1.;
        let (matrix, width, height) = make_grid_from2d_vec(&matrix);

        let res = isobands(
            &matrix,
            &vec![lower_band, lower_band + bandwidth],
            false,
            width,
            height,
        )
        .unwrap();
        assert_eq!(
            res[0].0,
            vec![
                vec![
                    Pt(0.25, 1.),
                    Pt(1., 0.25),
                    Pt(2., 0.25),
                    Pt(2.75, 1.),
                    Pt(2., 1.75),
                    Pt(1., 1.75),
                    Pt(0.25, 1.),
                ],
                vec![
                    Pt(0., 1.),
                    Pt(1., 2.),
                    Pt(1., 2.),
                    Pt(2., 2.),
                    Pt(3., 2.),
                    Pt(3., 1.),
                    Pt(3., 1.),
                    Pt(2., 0.),
                    Pt(2., 0.),
                    Pt(1., 0.),
                    Pt(0., 0.),
                    Pt(0., 1.),
                ]
            ]
        );
    }

    #[test]
    fn isobands_example() {
        let matrix = vec![
            vec![18., 13., 10., 9., 10., 13., 18.],
            vec![13., 8., 5., 4., 5., 8., 13.],
            vec![10., 5., 2., 1., 2., 5., 10.],
            vec![9., 4., 1., 12., 1., 4., 9.],
            vec![10., 5., 2., 1., 2., 5., 10.],
            vec![13., 8., 5., 4., 5., 8., 13.],
            vec![18., 13., 10., 9., 10., 13., 18.],
            vec![18., 13., 10., 9., 10., 13., 18.],
        ];
        let (matrix, width, height) = make_grid_from2d_vec(&matrix);
        let lower_band = 4.5;
        let bandwidth = 4.5;

        let res = isobands(
            &matrix,
            &vec![lower_band, lower_band + bandwidth],
            false,
            width,
            height,
        )
        .unwrap();
        assert_eq!(
            res[0].0,
            vec![
                vec![
                    Pt(1., 0.8),
                    Pt(0.8, 1.),
                    Pt(0.2, 2.),
                    Pt(0., 3.),
                    Pt(0., 3.),
                    Pt(0., 3.),
                    Pt(0.2, 4.),
                    Pt(0.8, 5.),
                    Pt(1., 5.2),
                    Pt(2., 5.8),
                    Pt(3., 6.),
                    Pt(3., 7.),
                    Pt(3., 7.),
                    Pt(3., 7.),
                    Pt(3., 6.),
                    Pt(4., 5.8),
                    Pt(5., 5.2),
                    Pt(5.2, 5.),
                    Pt(5.8, 4.),
                    Pt(6., 3.),
                    Pt(6., 3.),
                    Pt(6., 3.),
                    Pt(5.8, 2.),
                    Pt(5.2, 1.),
                    Pt(5., 0.8),
                    Pt(4., 0.2),
                    Pt(3., 0.),
                    Pt(3., 0.),
                    Pt(3., 0.),
                    Pt(2., 0.2),
                    Pt(1., 0.8),
                ],
                vec![
                    Pt(0.9, 3.),
                    Pt(1., 2.5),
                    Pt(1.1666666666666667, 2.),
                    Pt(2., 1.1666666666666667),
                    Pt(2.5, 1.),
                    Pt(3., 0.9),
                    Pt(3.5, 1.),
                    Pt(4., 1.1666666666666667),
                    Pt(4.833333333333333, 2.),
                    Pt(5., 2.5),
                    Pt(5.1, 3.),
                    Pt(5., 3.5),
                    Pt(4.833333333333333, 4.),
                    Pt(4., 4.833333333333333),
                    Pt(3.5, 5.),
                    Pt(3., 5.1),
                    Pt(2.5, 5.),
                    Pt(2., 4.833333333333333),
                    Pt(1.1666666666666667, 4.),
                    Pt(1., 3.5),
                    Pt(0.9, 3.),
                ],
                vec![
                    Pt(2.7272727272727275, 3.),
                    Pt(3., 2.7272727272727275),
                    Pt(3.2727272727272725, 3.),
                    Pt(3., 3.2727272727272725),
                    Pt(2.7272727272727275, 3.),
                ],
                vec![
                    Pt(3., 2.3181818181818183),
                    Pt(2.3181818181818183, 3.),
                    Pt(3., 3.6818181818181817),
                    Pt(3.6818181818181817, 3.),
                    Pt(3., 2.3181818181818183),
                ],
            ]
        );
    }

    #[test]
    fn isobands_original_code_issue_6_3_5() {
        let matrix = vec![
            vec![1., 1., 1., 1., 1., 1., 1.],
            vec![1., 5., 5., 5., 5., 5., 1.],
            vec![1., 5., 15., 15., 15., 5., 1.],
            vec![1., 5., 10., 10., 10., 5., 1.],
            vec![1., 5., 5., 5., 5., 5., 1.],
            vec![1., 1., 1., 1., 1., 1., 1.],
        ];
        let (matrix, width, height) = make_grid_from2d_vec(&matrix);

        let lower_band = 3.;
        let bandwidth = 2.;

        let res = isobands(
            &matrix,
            &vec![lower_band, lower_band + bandwidth],
            false,
            width,
            height,
        )
        .unwrap();
        assert_eq!(
            res[0].0,
            vec![
                vec![
                    Pt(1.0, 0.5),
                    Pt(0.5, 1.0),
                    Pt(0.5, 2.0),
                    Pt(0.5, 3.0),
                    Pt(0.5, 4.0),
                    Pt(1.0, 4.5),
                    Pt(2.0, 4.5),
                    Pt(3.0, 4.5),
                    Pt(4.0, 4.5),
                    Pt(5.0, 4.5),
                    Pt(5.5, 4.0),
                    Pt(5.5, 3.0),
                    Pt(5.5, 2.0),
                    Pt(5.5, 1.0),
                    Pt(5.0, 0.5),
                    Pt(4.0, 0.5),
                    Pt(3.0, 0.5),
                    Pt(2.0, 0.5),
                    Pt(1.0, 0.5)
                ],
                vec![
                    Pt(1.0, 2.0),
                    Pt(2.0, 1.0),
                    Pt(3.0, 1.0),
                    Pt(4.0, 1.0),
                    Pt(5.0, 2.0),
                    Pt(5.0, 3.0),
                    Pt(4.0, 4.0),
                    Pt(3.0, 4.0),
                    Pt(2.0, 4.0),
                    Pt(1.0, 3.0),
                    Pt(1.0, 2.0)
                ]
            ]
        );
    }

    #[test]
    fn isobands_original_code_issue_6_5_7() {
        let matrix = vec![
            vec![1., 1., 1., 1., 1., 1., 1.],
            vec![1., 5., 5., 5., 5., 5., 1.],
            vec![1., 5., 15., 15., 15., 5., 1.],
            vec![1., 5., 10., 10., 10., 5., 1.],
            vec![1., 5., 5., 5., 5., 5., 1.],
            vec![1., 1., 1., 1., 1., 1., 1.],
        ];
        let (matrix, width, height) = make_grid_from2d_vec(&matrix);

        let lower_band = 5.;
        let bandwidth = 2.;

        let res = isobands(
            &matrix,
            &vec![lower_band, lower_band + bandwidth],
            false,
            width,
            height,
        )
        .unwrap();
        assert_eq!(
            res[0].0,
            vec![
                vec![
                    Pt(1.0, 1.0),
                    Pt(1.0, 1.0),
                    Pt(1.0, 2.0),
                    Pt(1.0, 3.0),
                    Pt(1.0, 4.0),
                    Pt(1.0, 4.0),
                    Pt(2.0, 4.0),
                    Pt(3.0, 4.0),
                    Pt(4.0, 4.0),
                    Pt(5.0, 4.0),
                    Pt(5.0, 4.0),
                    Pt(5.0, 3.0),
                    Pt(5.0, 2.0),
                    Pt(5.0, 1.0),
                    Pt(5.0, 1.0),
                    Pt(4.0, 1.0),
                    Pt(3.0, 1.0),
                    Pt(2.0, 1.0),
                    Pt(1.0, 1.0)
                ],
                vec![
                    Pt(1.2, 2.0),
                    Pt(2.0, 1.2),
                    Pt(3.0, 1.2),
                    Pt(4.0, 1.2),
                    Pt(4.8, 2.0),
                    Pt(4.6, 3.0),
                    Pt(4.0, 3.6),
                    Pt(3.0, 3.6),
                    Pt(2.0, 3.6),
                    Pt(1.4, 3.0),
                    Pt(1.2, 2.0)
                ]
            ]
        );
    }

    #[test]
    fn isobands_multiple_bands() {
        let matrix = vec![
            vec![1., 1., 1., 1., 1., 1., 1.],
            vec![1., 5., 5., 5., 5., 5., 1.],
            vec![1., 5., 15., 15., 15., 5., 1.],
            vec![1., 5., 10., 10., 10., 5., 1.],
            vec![1., 5., 5., 5., 5., 5., 1.],
            vec![1., 1., 1., 1., 1., 1., 1.],
        ];
        let (matrix, width, height) = make_grid_from2d_vec(&matrix);

        let intervals = vec![3., 5., 7.];

        let res = isobands(&matrix, &intervals, false, width, height).unwrap();

        assert_eq!(res.len(), 2);

        assert_eq!(
            res[0].0,
            vec![
                vec![
                    Pt(0.9999750000000001, 1.0),
                    Pt(1.0, 0.9999750000000001),
                    Pt(2.0, 0.9999750000000001),
                    Pt(3.0, 0.9999750000000001),
                    Pt(4.0, 0.9999750000000001),
                    Pt(5.0, 0.9999750000000001),
                    Pt(5.000025, 1.0),
                    Pt(5.000025, 2.0),
                    Pt(5.000025, 3.0),
                    Pt(5.000025, 4.0),
                    Pt(5.0, 4.000025),
                    Pt(4.0, 4.000025),
                    Pt(3.0, 4.000025),
                    Pt(2.0, 4.000025),
                    Pt(1.0, 4.000025),
                    Pt(0.9999750000000001, 4.0),
                    Pt(0.9999750000000001, 3.0),
                    Pt(0.9999750000000001, 2.0),
                    Pt(0.9999750000000001, 1.0)
                ],
                vec![
                    Pt(1.0, 0.5),
                    Pt(0.5, 1.0),
                    Pt(0.5, 2.0),
                    Pt(0.5, 3.0),
                    Pt(0.5, 4.0),
                    Pt(1.0, 4.5),
                    Pt(2.0, 4.5),
                    Pt(3.0, 4.5),
                    Pt(4.0, 4.5),
                    Pt(5.0, 4.5),
                    Pt(5.5, 4.0),
                    Pt(5.5, 3.0),
                    Pt(5.5, 2.0),
                    Pt(5.5, 1.0),
                    Pt(5.0, 0.5),
                    Pt(4.0, 0.5),
                    Pt(3.0, 0.5),
                    Pt(2.0, 0.5),
                    Pt(1.0, 0.5)
                ]
            ]
        );
        assert_eq!(res[0].1, 3.0);
        assert_eq!(res[0].2, 5.0);

        assert_eq!(
            res[1].0,
            vec![
                vec![
                    Pt(1.0, 1.0),
                    Pt(1.0, 1.0),
                    Pt(1.0, 2.0),
                    Pt(1.0, 3.0),
                    Pt(1.0, 4.0),
                    Pt(1.0, 4.0),
                    Pt(2.0, 4.0),
                    Pt(3.0, 4.0),
                    Pt(4.0, 4.0),
                    Pt(5.0, 4.0),
                    Pt(5.0, 4.0),
                    Pt(5.0, 3.0),
                    Pt(5.0, 2.0),
                    Pt(5.0, 1.0),
                    Pt(5.0, 1.0),
                    Pt(4.0, 1.0),
                    Pt(3.0, 1.0),
                    Pt(2.0, 1.0),
                    Pt(1.0, 1.0)
                ],
                vec![
                    Pt(1.2, 2.0),
                    Pt(2.0, 1.2),
                    Pt(3.0, 1.2),
                    Pt(4.0, 1.2),
                    Pt(4.8, 2.0),
                    Pt(4.6, 3.0),
                    Pt(4.0, 3.6),
                    Pt(3.0, 3.6),
                    Pt(2.0, 3.6),
                    Pt(1.4, 3.0),
                    Pt(1.2, 2.0)
                ]
            ]
        );
        assert_eq!(res[1].1, 5.0);
        assert_eq!(res[1].2, 7.0);
    }

    #[test]
    /// Test that isobands returns the same result when using a quadtree or not (simple dataset)
    fn isobands_simple_same_with_quadtree() {
        let matrix = vec![
            vec![1., 1., 1., 0.],
            vec![1., 5., 5., 1.],
            vec![0., 1., 1., 1.],
        ];
        let (matrix, width, height) = make_grid_from2d_vec(&matrix);
        let lower_band = 1.;
        let bandwidth = 1.;

        let res1 = isobands(
            &matrix,
            &[lower_band, lower_band + bandwidth],
            false,
            width,
            height,
        )
        .unwrap();
        let res2 = isobands(
            &matrix,
            &[lower_band, lower_band + bandwidth],
            true,
            width,
            height,
        )
        .unwrap();

        assert_eq!(res1, res2);
    }

    #[test]
    /// Test that isobands returns the same result when using a quadtree or not (volcano dataset)
    fn isobands_volcano_same_with_quadtree() {
        let volcano_str = include_str!("../tests/fixtures/volcano.json");
        let raw_data: serde_json::Value = serde_json::from_str(volcano_str).unwrap();
        let matrix: Vec<f64> = raw_data["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();
        let h = raw_data["height"].as_u64().unwrap() as usize;
        let w = raw_data["width"].as_u64().unwrap() as usize;

        let intervals = [
            90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150., 155., 160.,
            165., 170., 175., 180., 185., 190., 195., 200.,
        ];

        let res1 = isobands(&matrix, &intervals, false, w, h).unwrap();
        let res2 = isobands(&matrix, &intervals, true, w, h).unwrap();

        assert_eq!(res1, res2);
    }

    #[test]
    /// Test that isobands returns the same result when using a quadtree or not (dataset from https://observablehq.com/@mthh/stewarts-potentials-on-the-gpu)
    fn isobands_pot_pop_same_with_quadtree() {
        let volcano_str = include_str!("../tests/fixtures/pot_pop_fr.json");
        let raw_data: serde_json::Value = serde_json::from_str(volcano_str).unwrap();
        let matrix: Vec<f64> = raw_data["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();
        let h = raw_data["height"].as_u64().unwrap() as usize;
        let w = raw_data["width"].as_u64().unwrap() as usize;

        let intervals = [
            0.001, 105483.25, 527416.25, 1054832.5, 2109665., 3164497.5, 4219330., 5274162.5,
            6328995., 7383827.5, 8438660., 9704459., 10548326.,
        ];

        let res1 = isobands(&matrix, &intervals, false, w, h).unwrap();
        let res2 = isobands(&matrix, &intervals, true, w, h).unwrap();

        assert_eq!(res1, res2);
    }
}
