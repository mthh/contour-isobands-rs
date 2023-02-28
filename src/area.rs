use crate::isobands::Pt;

pub fn area(ring: &[Pt]) -> f64 {
    let mut i = 0;
    let n = ring.len() - 1;
    let mut area = ring[n - 1].1 * ring[0].0 - ring[n - 1].0 * ring[0].1;
    while i < n {
        i += 1;
        area += ring[i - 1].1 * ring[i].0 - ring[i - 1].0 * ring[i].1;
    }
    area
}

pub fn contains(ring: &[Pt], hole: &[Pt]) -> bool {
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

fn ring_contains(ring: &[Pt], point: &Pt) -> bool {
    let x = point.0;
    let y = point.1;
    let n = ring.len();
    // let mut contains = -1;
    let mut contains = false;
    let mut j = n - 1;
    for i in 0..n {
        let pi = &ring[i];
        let xi = pi.0;
        let yi = pi.1;
        let pj = &ring[j];
        let xj = pj.0;
        let yj = pj.1;
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

fn segment_contains(a: &Pt, b: &Pt, c: &Pt) -> bool {
    if collinear(a, b, c) {
        if (a.0 - b.0).abs() < f64::EPSILON {
            within(a.1, c.1, b.1)
        } else {
            within(a.0, c.0, b.0)
        }
    } else {
        false
    }
}

fn collinear(a: &Pt, b: &Pt, c: &Pt) -> bool {
    ((b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1)).abs() < f64::EPSILON
}

fn within(p: f64, q: f64, r: f64) -> bool {
    p <= q && q <= r || r <= q && q <= p
}
