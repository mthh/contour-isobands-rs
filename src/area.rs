use geo_types::Coord;

pub(crate) fn area(ring: &[Coord<f64>]) -> f64 {
    let mut i = 0;
    let n = ring.len() - 1;
    let mut area = ring[n - 1].y * ring[0].x - ring[n - 1].x * ring[0].y;
    while i < n {
        i += 1;
        area += ring[i - 1].y * ring[i].x - ring[i - 1].x * ring[i].y;
    }
    area
}
