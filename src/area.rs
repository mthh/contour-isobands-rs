use geo_types::Coord;

/// Compute signed area of a ring.
/// We expect the ring to be closed, i.e. the first and last points are the same
/// (this is not checked because we already know it's true due to the way we
/// construct the rings in the trace_band_paths function).
/// Note that in the shoelace formula you need to divide this result by 2 to get the actual area.
/// Here we skip this division because we only use this area formula to calculate the winding
/// order of polygons and to compare their relative sizes.
pub(crate) fn area(ring: &[Coord<f64>]) -> f64 {
    let n = ring.len();
    if n < 3 {
        return 0.;
    }
    let mut area = ring[n - 1].y * ring[0].x - ring[n - 1].x * ring[0].y;
    for pts in ring.windows(2) {
        area += pts[0].y * pts[1].x - pts[0].x * pts[1].y;
    }
    area
}
