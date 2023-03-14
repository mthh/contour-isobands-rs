# Contour-isobands-rs

[![Build status GitHub Actions](https://github.com/mthh/contour-isobands-rs/actions/workflows/build_test_ubuntu.yml/badge.svg)](https://github.com/mthh/contour-isobands-rs/actions/workflows/build_test_ubuntu.yml)
[![Docs.rs version](https://docs.rs/contour-isobands/badge.svg)](https://docs.rs/contour-isobands/)

Compute isobands *(i.e. contour polygons which enclose all the points of a grid included
between two given values)* by applying marching squares to an array of values.

![](https://raw.githubusercontent.com/mthh/contour-isobands-rs/main/illustration.png)

### Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
contour-isobands-rs = "0.1.0"
```

Then, you can use the `ContourBuilder` to compute isobands:

```rust
use contour_isobands_rs::{ContourBuilder, Band};

let values = vec![
    1., 1., 1., 1., 1., 1., 1.,
    1., 5., 5., 5., 5., 5., 1.,
    1., 5., 15., 15., 15., 5., 1.,
    1., 5., 10., 10., 10., 5., 1.,
    1., 5., 5., 5., 5., 5., 1.,
    1., 1., 1., 1., 1., 1., 1.,
];

// These intervals will compute 3 bands:
// - the first one will contain all points between 1 (included) and 5 (excluded)
// - the second one will contain all points between 5 (included) and 7 (excluded)
// - the third one will contain all points between 7 (included) and 15 (included)
let intervals = vec![1, 5, 7, 15];

let result: Vec<Band> = ContourBuilder::new(7, 6)
    .use_quadtree(true)
    .contours(&values, &intervals)?;

assert_eq!(result.len(), 3);
```

Each `Band` struct contains a geometry (`MultiPolygon<f64>`) and the minimum and maximum values of the band.
It can be serialized to geojson using the `geojson` feature:

```rust
use contour_isobands_rs::{ContourBuilder, Band};
use geojson::{Feature, FeatureCollection};

let values = vec![
    1., 1., 1., 1., 1., 1., 1.,
    1., 5., 5., 5., 5., 5., 1.,
    1., 5., 15., 15., 15., 5., 1.,
    1., 5., 10., 10., 10., 5., 1.,
    1., 5., 5., 5., 5., 5., 1.,
    1., 1., 1., 1., 1., 1., 1.,
];

let intervals = vec![1, 5, 7, 15];

let result = ContourBuilder::new(7, 6)
    .use_quadtree(true)
    .contours(&values, &intervals)?;
    
let features = result.iter()
    .map(|band| band.to_geojson())
    .collect::<Vec<geojson::Feature>>();

let geojson_string = GeoJson::from(
    FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    }).to_string();
```

Note that you can specify the coordinates of the grid and the distance between points (on x- and y-axis)
using the `x_origin`, `y_origin`, `x_step` and `y_step` parameters of the `ContourBuilder` constructor :

```rust
let result: Vec<Band> = ContourBuilder::new(7, 6)
    .x_origin(-6.141583932015321)
    .y_origin(51.7591540368963)
    .x_step(-0.119)
    .y_step(0.09)
    .use_quadtree(true)
    .contours(&values, &intervals)?;
```

### Difference with the [contour](https://crates.io/crates/contour) crate (from [`mthh/contour-rs`](https://github.com/mthh/contour-rs) repository)

While the [contour](https://crates.io/crates/contour) crate computes *__isolines__*
(cf. [wikipedia:Marching_squares](https://en.wikipedia.org/wiki/Marching_squares)) and
their corresponding polygons *(i.e. polygons that contain all points above the threshold defined for a given isoline)*,
`contour-isobands-rs` computes *__isobands__* (cf. [wikipedia:Marching_squares#Isobands](https://en.wikipedia.org/wiki/Marching_squares#Isobands)) and their
corresponding polygons *(i.e. contour polygons that contain all points between a minimum and a maximum bound)*.

Depending on the desired use of the result, this `contour-isobands` crate may be more suitable than the `contour` crate (for example to visualize results with an opacity lower than 100%).

### Licence

Since this is mostly a port of [https://github.com/RaumZeit/MarchingSquares.js](https://github.com/RaumZeit/MarchingSquares.js) which is licenced under the Affero General Public License v3.0, this project is also licenced under the Affero General Public License v3.0.
See the [LICENSE](LICENSE) file for details.