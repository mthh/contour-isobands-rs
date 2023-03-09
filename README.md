# Contour-isobands-rs

Compute isobands *(i.e. contour polygons which enclose all the points of a grid included
between two given values)* by applying marching squares to an array of values.

### Difference with [`mthh/contour-rs`](https://github.com/mthh/contour-rs)
While [`mthh/contour-rs`](https://github.com/mthh/contour-rs) computes isolines
(cf. [wikipedia:Marching_squares#Disambiguation_of_saddle_points](https://en.wikipedia.org/wiki/Marching_squares#Disambiguation_of_saddle_points)) and
their corresponding polygons *(i.e. polygons that contain all points above the threshold defined for a given isoline)*,
`contour-isobands-rs` computes isobands (cf. [wikipedia:Marching_squares#Isobands](https://en.wikipedia.org/wiki/Marching_squares#Isobands)) and their
corresponding polygons *(i.e. contour polygons that contain all points between a minimum and a maximum bound)*.

![](illustration.png)

### Usage

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
    .contours(&values, &intervals);

assert_eq!(result.len(), 3);
```

### Status

This library is still WIP, but it should be published on crates.io soon.

Current status / roadmap is as follows:

- [x] All the isobands code (from [RaumZeit/MarchingSquares.js](https://github.com/RaumZeit/MarchingSquares.js)) is ported and tests are passing
- [x] Implement a spatial index to filter calls to `prepare_cell` (although it only yields improved performance for large grids / when using numerous thresholds)
- [x] Return contours using geo_types primitives and propose GeoJSON serialisation
- [x] API is polished enough (and close to contour-rs API)
- [ ] Rename [`contour-rs`](https://github.com/mthh/contour-rs)  to `contour-isolines-rs` to distinguish it from this library (`contour-isobands-rs`) ?
- [ ] Publish on crates.io
- [ ] Make a WASM library of it (WIP - https://github.com/mthh/contour-isobands-wasm)
- [ ] Make a WASM example (WIP - https://github.com/mthh/contour-isobands-wasm)

### Licence

Since this is a port from [https://github.com/RaumZeit/MarchingSquares.js](https://github.com/RaumZeit/MarchingSquares.js) which is licenced under the Affero General Public License v3.0, this project is also licenced under the Affero General Public License v3.0.
See the [LICENSE](LICENSE) file for details.