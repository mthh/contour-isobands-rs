#![feature(test)]
extern crate contour_isobands;
extern crate test;

use contour_isobands::isobands;
use contour_isobands::ContourBuilder;
use test::{black_box, Bencher};

#[bench]
fn bench_isobands_no_quadtree_pot_pop_fr(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/pot_pop_fr.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            isobands(
                &matrix,
                &[
                    0.001, 105483.25, 527416.25, 1054832.5, 2109665., 3164497.5, 4219330.,
                    5274162.5, 6328995., 7383827.5, 8438660., 9704459., 10548326.,
                ],
                false,
                w,
                h,
                false,
            )
            .unwrap(),
        )
    });
}

#[bench]
fn bench_isobands_quadtree_pot_pop_fr(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/pot_pop_fr.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            isobands(
                &matrix,
                &[
                    0.001, 105483.25, 527416.25, 1054832.5, 2109665., 3164497.5, 4219330.,
                    5274162.5, 6328995., 7383827.5, 8438660., 9704459., 10548326.,
                ],
                true,
                w,
                h,
                false,
            )
            .unwrap(),
        )
    });
}

#[bench]
fn bench_isobands_no_quadtree_volcano(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            isobands(
                &matrix,
                &[
                    90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150.,
                    155., 160., 165., 170., 175., 180., 185., 190., 195., 200.,
                ],
                false,
                w,
                h,
                false,
            )
            .unwrap(),
        )
    });
}

#[bench]
fn bench_isobands_quadtree_volcano(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            isobands(
                &matrix,
                &[
                    90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150.,
                    155., 160., 165., 170., 175., 180., 185., 190., 195., 200.,
                ],
                true,
                w,
                h,
                false,
            )
            .unwrap(),
        )
    });
}

#[bench]
fn bench_contourbuilder_volcano_no_quadtree_without_xy_step_xy_origin(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            ContourBuilder::new(w, h)
                .use_quad_tree(false)
                .contours(
                    &matrix,
                    &[
                        90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150.,
                        155., 160., 165., 170., 175., 180., 185., 190., 195., 200.,
                    ],
                )
                .unwrap(),
        )
    });
}

#[bench]
fn bench_contourbuilder_volcano_no_quadtree_with_xy_step_xy_origin(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            ContourBuilder::new(w, h)
                .x_origin(100.)
                .y_origin(100.)
                .x_step(15.)
                .y_step(15.)
                .use_quad_tree(false)
                .contours(
                    &matrix,
                    &[
                        90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150.,
                        155., 160., 165., 170., 175., 180., 185., 190., 195., 200.,
                    ],
                )
                .unwrap(),
        )
    });
}

#[bench]
#[cfg(feature = "parallel")]
fn bench_contourbuilder_volcano_no_quadtree_without_xy_step_xy_origin_parallel(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            ContourBuilder::new(w, h)
                .use_quad_tree(false)
                .par_contours(
                    &matrix,
                    &[
                        90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150.,
                        155., 160., 165., 170., 175., 180., 185., 190., 195., 200.,
                    ],
                )
                .unwrap(),
        )
    });
}

#[bench]
#[cfg(feature = "parallel")]
fn bench_contourbuilder_volcano_no_quadtree_with_xy_step_xy_origin_parallel(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            ContourBuilder::new(w, h)
                .x_origin(100.)
                .y_origin(100.)
                .x_step(15.)
                .y_step(15.)
                .use_quad_tree(false)
                .par_contours(
                    &matrix,
                    &[
                        90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150.,
                        155., 160., 165., 170., 175., 180., 185., 190., 195., 200.,
                    ],
                )
                .unwrap(),
        )
    });
}

#[bench]
#[cfg(feature = "parallel")]
fn bench_contourbuilder_volcano_quadtree_without_xy_step_xy_origin_parallel(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            ContourBuilder::new(w, h)
                .use_quad_tree(true)
                .par_contours(
                    &matrix,
                    &[
                        90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150.,
                        155., 160., 165., 170., 175., 180., 185., 190., 195., 200.,
                    ],
                )
                .unwrap(),
        )
    });
}

#[bench]
#[cfg(feature = "parallel")]
fn bench_contourbuilder_volcano_quadtree_with_xy_step_xy_origin_parallel(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            ContourBuilder::new(w, h)
                .x_origin(100.)
                .y_origin(100.)
                .x_step(15.)
                .y_step(15.)
                .use_quad_tree(true)
                .par_contours(
                    &matrix,
                    &[
                        90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150.,
                        155., 160., 165., 170., 175., 180., 185., 190., 195., 200.,
                    ],
                )
                .unwrap(),
        )
    });
}

#[bench]
fn bench_contourbuilder_volcano_quadtree_without_xy_step_xy_origin(b: &mut Bencher) {
    let data_str = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    b.iter(|| {
        black_box(
            ContourBuilder::new(w, h)
                .use_quad_tree(true)
                .contours(
                    &matrix,
                    &[
                        90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150.,
                        155., 160., 165., 170., 175., 180., 185., 190., 195., 200.,
                    ],
                )
                .unwrap(),
        )
    });
}