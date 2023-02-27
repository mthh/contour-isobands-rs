# Contour-isobands-rs


### Status

This library is WIP, but it should be published on crates.io soon.

[x] All the isobands code is ported  
[x] Tests are passing  
[ ] Return contours using geo_types primitives and propose GeoJSON serialisation  
[ ] API is polished enough (and close to contour-rs API)  
[ ] Rename contour-rs to contour-isolines-rs to distinguish it from this library (contour-isobands-rs)  
[ ] Publish on crates.io  
[ ] Use a spatial index to filter calls to `prepare_cell`  
[ ] Make a WASM example

### Licence

Since this is a port from [https://github.com/RaumZeit/MarchingSquares.js](https://github.com/RaumZeit/MarchingSquares.js) which is licenced under the Affero General Public License v3.0, this project is also licenced under the Affero General Public License v3.0.
See the [LICENSE](LICENSE) file for details.