use geo_types::Coord;

pub(crate) fn contains(ring: &[Coord<f64>], hole: &[Coord<f64>]) -> bool {
    let mut i = 0;
    let n = hole.len();
    let mut c;
    while i < n {
        c = ring_contains(ring, &hole[i]);
        if c {
            return c;
        }
        i += 1;
    }
    false
}

fn ring_contains(ring: &[Coord<f64>], point: &Coord<f64>) -> bool {
    let x = point.x;
    let y = point.y;
    let n = ring.len();
    // let mut contains = -1;
    let mut contains = false;
    let mut j = n - 1;
    for i in 0..n {
        let pi = &ring[i];
        let xi = pi.x;
        let yi = pi.y;
        let pj = &ring[j];
        let xj = pj.x;
        let yj = pj.y;
        if segment_contains(pi, pj, point) {
            return false;
        }
        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
            // contains = -contains;
            contains = !contains;
        }
        j = i;
    }
    contains
}

fn segment_contains(a: &Coord<f64>, b: &Coord<f64>, c: &Coord<f64>) -> bool {
    if collinear(a, b, c) {
        if (a.x - b.x).abs() < f64::EPSILON {
            within(a.y, c.y, b.y)
        } else {
            within(a.x, c.x, b.x)
        }
    } else {
        false
    }
}

fn collinear(a: &Coord<f64>, b: &Coord<f64>, c: &Coord<f64>) -> bool {
    ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs() < f64::EPSILON
}

fn within(p: f64, q: f64, r: f64) -> bool {
    p <= q && q <= r || r <= q && q <= p
}
