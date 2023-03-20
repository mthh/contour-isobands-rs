# Changelog

### 0.3.0 (2023-03-20)

- Add new `par_contours` method to `ContourBuilder` to generate contours in parallel (only available with the `parallel` feature flag).

- Fix assignment of interior rings when a polygon with a hole is contained in the hole of another polygon (on the same band).

- Use more idiomatic code for `area` and `contains` functions.

- Don't store `cval` on `Cell` struct.

- Avoid pulling and compiling serde_json when the `geojson` feature is not enabled.

- Improve documentation and README.

- Add executable example.


### 0.2.0 (2023-03-14)

- Ensure correct winding order of polygon rings.

- Remove repeated point in polygon rings if any.

- Avoid cloning some values when reconstructing polygons.


### 0.1.0 (2023-03-14)

First release.