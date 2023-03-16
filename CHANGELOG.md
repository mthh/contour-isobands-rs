# Changelog

### Unreleased

- Use more idiomatic code for `area` and `contains` functions.

- Avoid pulling and compiling serde_json when the `geojson` feature is not enabled.

- Improve documentation and README.

- Add executable example.


### 0.2.0 (2023-03-14)

- Ensure correct winding order of polygon rings.

- Remove repeated point in polygon rings if any.

- Avoid cloning some values when reconstructing polygons.


### 0.1.0 (2023-03-14)

First release.