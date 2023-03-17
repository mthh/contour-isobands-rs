use crate::errors::{new_error, ErrorKind, Result};
use crate::grid::BorrowedGrid;
use crate::isobands::{Cell, Edge, EnterType, MoveInfo, Pt, Settings};
use rustc_hash::FxHashMap;
// use lazy_static::lazy_static;
// use rustc_hash::FxHasher;
// use std::collections::HashMap;
// use std::hash::BuildHasherDefault;

fn interpolate_linear_ab(a: f64, b: f64, v0: f64, v1: f64) -> f64 {
    let (v0, v1) = if v0 > v1 { (v1, v0) } else { (v0, v1) };

    if a < b {
        if a < v0 {
            (v0 - a) / (b - a)
        } else {
            (v1 - a) / (b - a)
        }
    } else if a > v1 {
        (a - v1) / (a - b)
    } else {
        (a - v0) / (a - b)
    }
}

fn interpolate_linear_a(a: f64, b: f64, min_v: f64, max_v: f64) -> f64 {
    if a < b {
        (min_v - a) / (b - a)
    } else {
        (a - max_v) / (a - b)
    }
}

fn interpolate_linear_b(a: f64, b: f64, min_v: f64, max_v: f64) -> f64 {
    if a < b {
        (max_v - a) / (b - a)
    } else {
        (a - min_v) / (a - b)
    }
}

fn compute_center_average(bl: f64, br: f64, tl: f64, tr: f64, min_v: f64, max_v: f64) -> u8 {
    let average = (bl + br + tl + tr) / 4.;
    if average > max_v {
        2
    } else if average < min_v {
        0
    } else {
        1
    }
}

/// Below are lookup for shapes, ported from https://github.com/RaumZeit/MarchingSquares.js/blob/master/src/isobands.js

fn square(_cell: &mut Cell, _opt: &Settings) {
    // This is a no-op due to how we are tracing the polygons
    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., 1.),
    //         Pt(1., 1.),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn triangle_bl(cell: &mut Cell, opt: &Settings) {
    let bottomleft = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let leftbottom = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::LB,
        Edge {
            path: [Pt(0., leftbottom), Pt(bottomleft, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TL,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., leftbottom),
    //         Pt(bottomleft, 0.),
    //         Pt(0., 0.),
    //     ]
    // );
}

fn triangle_br(cell: &mut Cell, opt: &Settings) {
    let bottomright = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BR,
        Edge {
            path: [Pt(bottomright, 0.), Pt(1., rightbottom)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LB,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomright, 0.),
    //         Pt(1., rightbottom),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn triangle_tr(cell: &mut Cell, opt: &Settings) {
    let righttop = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let topright = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::RT,
        Edge {
            path: [Pt(1., righttop), Pt(topright, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(1., righttop),
    //         Pt(topright, 1.),
    //         Pt(1., 1.),
    //     ]
    // );
}

fn triangle_tl(cell: &mut Cell, opt: &Settings) {
    let topleft = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::TL,
        Edge {
            path: [Pt(topleft, 1.), Pt(0., lefttop)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RT,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., lefttop),
    //         Pt(0., 1.),
    //         Pt(topleft, 1.),
    //     ]
    // );
}

fn tetragon_t(cell: &mut Cell, opt: &Settings) {
    let righttop = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::RT,
        Edge {
            path: [Pt(1., righttop), Pt(0., lefttop)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RT,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., lefttop),
    //         Pt(0., 1.),
    //         Pt(1., 1.),
    //         Pt(1., righttop),
    //     ]
    // );
}

fn tetragon_r(cell: &mut Cell, opt: &Settings) {
    let bottomright = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let topright = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BR,
        Edge {
            path: [Pt(bottomright, 0.), Pt(topright, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomright, 0.),
    //         Pt(topright, 1.),
    //         Pt(1., 1.),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn tetragon_b(cell: &mut Cell, opt: &Settings) {
    let leftbottom = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::LB,
        Edge {
            path: [Pt(0., leftbottom), Pt(1., rightbottom)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LB,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., leftbottom),
    //         Pt(1., rightbottom),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn tetragon_l(cell: &mut Cell, opt: &Settings) {
    let topleft = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let bottomleft = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::TL,
        Edge {
            path: [Pt(topleft, 1.), Pt(bottomleft, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TL,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(1., 0.),
    //         Pt(topleft, 1.),
    //         Pt(bottomleft, 0.),
    //     ]
    // );
}

fn tetragon_bl(cell: &mut Cell, opt: &Settings) {
    let bottomleft = interpolate_linear_a(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_b(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let leftbottom = interpolate_linear_a(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_b(cell.x0, cell.x3, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BL,
        Edge {
            path: [Pt(bottomleft, 0.), Pt(0., leftbottom)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RB,
            },
        },
    );

    cell.edges.insert(
        EnterType::LT,
        Edge {
            path: [Pt(0., lefttop), Pt(bottomright, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomleft, 0.),
    //         Pt(0., leftbottom),
    //         Pt(0., lefttop),
    //         Pt(bottomright, 0.),
    //     ]
    // );
}

fn tetragon_br(cell: &mut Cell, opt: &Settings) {
    let bottomleft = interpolate_linear_a(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_b(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_a(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_b(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BL,
        Edge {
            path: [Pt(bottomleft, 0.), Pt(1., righttop)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LT,
            },
        },
    );

    cell.edges.insert(
        EnterType::RB,
        Edge {
            path: [Pt(1., rightbottom), Pt(bottomright, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomleft, 0.),
    //         Pt(1., righttop),
    //         Pt(1., rightbottom),
    //         Pt(bottomright, 0.),
    //     ]
    // );
}

fn tetragon_tr(cell: &mut Cell, opt: &Settings) {
    let topleft = interpolate_linear_a(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let topright = interpolate_linear_b(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_b(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_a(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::RB,
        Edge {
            path: [Pt(1., rightbottom), Pt(topleft, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BL,
            },
        },
    );

    cell.edges.insert(
        EnterType::TR,
        Edge {
            path: [Pt(topright, 1.), Pt(1., righttop)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LT,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(1., rightbottom),
    //         Pt(topleft, 1.),
    //         Pt(topright, 1.),
    //         Pt(1., righttop),
    //     ]
    // );
}

fn tetragon_tl(cell: &mut Cell, opt: &Settings) {
    let topleft = interpolate_linear_a(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let topright = interpolate_linear_b(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_b(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let leftbottom = interpolate_linear_a(cell.x0, cell.x3, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::TR,
        Edge {
            path: [Pt(topright, 1.), Pt(0., leftbottom)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RB,
            },
        },
    );

    cell.edges.insert(
        EnterType::LT,
        Edge {
            path: [Pt(0., lefttop), Pt(topleft, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BL,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(topright, 1.),
    //         Pt(0., leftbottom),
    //         Pt(0., lefttop),
    //         Pt(topleft, 1.),
    //     ]
    // );
}

fn tetragon_lr(cell: &mut Cell, opt: &Settings) {
    let leftbottom = interpolate_linear_a(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_b(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_b(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_a(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::LT,
        Edge {
            path: [Pt(0., lefttop), Pt(1., righttop)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LT,
            },
        },
    );

    cell.edges.insert(
        EnterType::RB,
        Edge {
            path: [Pt(1., rightbottom), Pt(0., leftbottom)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RB,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., leftbottom),
    //         Pt(0., lefttop),
    //         Pt(1., righttop),
    //         Pt(1., rightbottom),
    //     ]
    // );
}

fn tetragon_tb(cell: &mut Cell, opt: &Settings) {
    let topleft = interpolate_linear_a(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let topright = interpolate_linear_b(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_b(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let bottomleft = interpolate_linear_a(cell.x0, cell.x1, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::TR,
        Edge {
            path: [Pt(topright, 1.), Pt(bottomright, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TR,
            },
        },
    );

    cell.edges.insert(
        EnterType::BL,
        Edge {
            path: [Pt(bottomleft, 0.), Pt(topleft, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BL,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomleft, 0.),
    //         Pt(topleft, 1.),
    //         Pt(topright, 1.),
    //         Pt(bottomright, 1.),
    //     ]
    // );
}

fn pentagon_tr(cell: &mut Cell, opt: &Settings) {
    let topleft = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::TL,
        Edge {
            path: [Pt(topleft, 1.), Pt(1., rightbottom)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LB,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., 1.),
    //         Pt(topleft, 1.),
    //         Pt(1., rightbottom),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn pentagon_tl(cell: &mut Cell, opt: &Settings) {
    let leftbottom = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topright = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::LB,
        Edge {
            path: [Pt(0., leftbottom), Pt(topright, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., leftbottom),
    //         Pt(topright, 1.),
    //         Pt(1., 1.),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn pentagon_br(cell: &mut Cell, opt: &Settings) {
    let bottomleft = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::RT,
        Edge {
            path: [Pt(1., righttop), Pt(bottomleft, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TL,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., 1.),
    //         Pt(1., 1.),
    //         Pt(1., righttop),
    //         Pt(bottomleft, 0.),
    //     ]
    // );
}

fn pentagon_bl(cell: &mut Cell, opt: &Settings) {
    let lefttop = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BR,
        Edge {
            path: [Pt(bottomright, 0.), Pt(0., lefttop)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RT,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., lefttop),
    //         Pt(0., 1.),
    //         Pt(1., 1.),
    //         Pt(1., 0.),
    //         Pt(bottomright, 0.),
    //     ]
    // );
}

fn pentagon_tr_rl(cell: &mut Cell, opt: &Settings) {
    let lefttop = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topleft = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_b(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_a(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::TL,
        Edge {
            path: [Pt(topleft, 1.), Pt(1., righttop)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LT,
            },
        },
    );

    cell.edges.insert(
        EnterType::RB,
        Edge {
            path: [Pt(1., rightbottom), Pt(0., lefttop)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RT,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., lefttop),
    //         Pt(0., 1.),
    //         Pt(topleft, 1.),
    //         Pt(1., righttop),
    //         Pt(1., rightbottom),
    //     ]
    // );
}

fn pentagon_rb_bt(cell: &mut Cell, opt: &Settings) {
    let righttop = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_b(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let bottomleft = interpolate_linear_a(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let topright = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::RT,
        Edge {
            path: [Pt(1., righttop), Pt(bottomright, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TR,
            },
        },
    );

    cell.edges.insert(
        EnterType::BL,
        Edge {
            path: [Pt(bottomleft, 0.), Pt(topright, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(topright, 1.),
    //         Pt(1., 1.),
    //         Pt(1., righttop),
    //         Pt(bottomright, 0.),
    //         Pt(bottomleft, 0.),
    //     ]
    // );
}

fn pentagon_bl_lr(cell: &mut Cell, opt: &Settings) {
    let bottomright = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let leftbottom = interpolate_linear_a(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_b(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BR,
        Edge {
            path: [Pt(bottomright, 0.), Pt(0., leftbottom)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RB,
            },
        },
    );

    cell.edges.insert(
        EnterType::LT,
        Edge {
            path: [Pt(0., lefttop), Pt(1., rightbottom)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LB,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomright, 0.),
    //         Pt(0., leftbottom),
    //         Pt(0., lefttop),
    //         Pt(1., rightbottom),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn pentagon_lt_tb(cell: &mut Cell, opt: &Settings) {
    let leftbottom = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topleft = interpolate_linear_a(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let topright = interpolate_linear_b(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let bottomleft = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::LB,
        Edge {
            path: [Pt(0., leftbottom), Pt(topleft, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BL,
            },
        },
    );

    cell.edges.insert(
        EnterType::TR,
        Edge {
            path: [Pt(topright, 1.), Pt(bottomleft, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TL,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., leftbottom),
    //         Pt(topleft, 1.),
    //         Pt(topright, 1.),
    //         Pt(bottomleft, 0.),
    //     ]
    // );
}

fn pentagon_bl_tb(cell: &mut Cell, opt: &Settings) {
    let lefttop = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topleft = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_b(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let bottomleft = interpolate_linear_a(cell.x0, cell.x1, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BL,
        Edge {
            path: [Pt(bottomleft, 0.), Pt(0., lefttop)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RT,
            },
        },
    );

    cell.edges.insert(
        EnterType::TL,
        Edge {
            path: [Pt(topleft, 1.), Pt(bottomright, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., lefttop),
    //         Pt(0., 1.),
    //         Pt(topleft, 1.),
    //         Pt(bottomright, 0.),
    //         Pt(bottomleft, 0.),
    //     ]
    // );
}

fn pentagon_lt_rl(cell: &mut Cell, opt: &Settings) {
    let leftbottom = interpolate_linear_a(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_b(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topright = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_ab(cell.x1, cell.x3, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::LT,
        Edge {
            path: [Pt(0., lefttop), Pt(topright, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BR,
            },
        },
    );

    cell.edges.insert(
        EnterType::RT,
        Edge {
            path: [Pt(1., righttop), Pt(0., leftbottom)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RB,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., leftbottom),
    //         Pt(0., lefttop),
    //         Pt(topright, 1.),
    //         Pt(1., 1.),
    //         Pt(1., righttop),
    //     ]
    // );
}

fn pentagon_tr_bt(cell: &mut Cell, opt: &Settings) {
    let topleft = interpolate_linear_a(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let topright = interpolate_linear_b(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BR,
        Edge {
            path: [Pt(bottomright, 0.), Pt(topleft, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BL,
            },
        },
    );

    cell.edges.insert(
        EnterType::TR,
        Edge {
            path: [Pt(topright, 1.), Pt(1., rightbottom)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LB,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(topleft, 1.),
    //         Pt(topright, 1.),
    //         Pt(1., rightbottom),
    //         Pt(1., 0.),
    //         Pt(bottomright, 0.),
    //     ]
    // );
}

fn pentagon_rb_lr(cell: &mut Cell, opt: &Settings) {
    let leftbottom = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_b(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_a(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let bottomleft = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::LB,
        Edge {
            path: [Pt(0., leftbottom), Pt(1., righttop)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LT,
            },
        },
    );

    cell.edges.insert(
        EnterType::RB,
        Edge {
            path: [Pt(1., rightbottom), Pt(bottomleft, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TL,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., leftbottom),
    //         Pt(1., righttop),
    //         Pt(1., rightbottom),
    //         Pt(bottomleft, 0.),
    //     ]
    // );
}

fn hexagon_lt_tr(cell: &mut Cell, opt: &Settings) {
    let leftbottom = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topleft = interpolate_linear_a(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let topright = interpolate_linear_b(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::LB,
        Edge {
            path: [Pt(0., leftbottom), Pt(topleft, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BL,
            },
        },
    );

    cell.edges.insert(
        EnterType::TR,
        Edge {
            path: [Pt(topright, 1.), Pt(1., rightbottom)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LB,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., leftbottom),
    //         Pt(topleft, 1.),
    //         Pt(topright, 1.),
    //         Pt(1., rightbottom),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn hexagon_bl_lt(cell: &mut Cell, opt: &Settings) {
    let bottomright = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let leftbottom = interpolate_linear_a(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_b(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topright = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BR,
        Edge {
            path: [Pt(bottomright, 0.), Pt(0., leftbottom)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RB,
            },
        },
    );

    cell.edges.insert(
        EnterType::LT,
        Edge {
            path: [Pt(0., lefttop), Pt(topright, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomright, 0.),
    //         Pt(0., leftbottom),
    //         Pt(0., lefttop),
    //         Pt(topright, 1.),
    //         Pt(1., 1.),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn hexagon_bl_rb(cell: &mut Cell, opt: &Settings) {
    let bottomleft = interpolate_linear_a(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_b(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BL,
        Edge {
            path: [Pt(bottomleft, 0.), Pt(0., lefttop)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RT,
            },
        },
    );

    cell.edges.insert(
        EnterType::RT,
        Edge {
            path: [Pt(1., righttop), Pt(bottomright, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomleft, 0.),
    //         Pt(0., lefttop),
    //         Pt(0., 1.),
    //         Pt(1., 1.),
    //         Pt(1., righttop),
    //         Pt(bottomright, 0.),
    //     ]
    // );
}

fn hexagon_tr_rb(cell: &mut Cell, opt: &Settings) {
    let bottomleft = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let topleft = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_b(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_a(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::TL,
        Edge {
            path: [Pt(topleft, 1.), Pt(1., righttop)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LT,
            },
        },
    );

    cell.edges.insert(
        EnterType::RB,
        Edge {
            path: [Pt(1., rightbottom), Pt(bottomleft, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TL,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., 1.),
    //         Pt(topleft, 1.),
    //         Pt(1., righttop),
    //         Pt(1., rightbottom),
    //         Pt(bottomleft, 0.),
    //     ]
    // );
}

fn hexagon_lt_rb(cell: &mut Cell, opt: &Settings) {
    let leftbottom = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topright = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let bottomleft = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::LB,
        Edge {
            path: [Pt(0., leftbottom), Pt(topright, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BR,
            },
        },
    );

    cell.edges.insert(
        EnterType::RT,
        Edge {
            path: [Pt(1., righttop), Pt(bottomleft, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TL,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., leftbottom),
    //         Pt(topright, 1.),
    //         Pt(1., 1.),
    //         Pt(1., righttop),
    //         Pt(bottomleft, 0.),
    //     ]
    // );
}

fn hexagon_bl_tr(cell: &mut Cell, opt: &Settings) {
    let bottomright = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topleft = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BR,
        Edge {
            path: [Pt(bottomright, 0.), Pt(0., lefttop)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RT,
            },
        },
    );

    cell.edges.insert(
        EnterType::TL,
        Edge {
            path: [Pt(topleft, 1.), Pt(1., rightbottom)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LB,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomright, 0.),
    //         Pt(0., lefttop),
    //         Pt(0., 1.),
    //         Pt(topleft, 1.),
    //         Pt(1., rightbottom),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn heptagon_tr(cell: &mut Cell, opt: &Settings) {
    let bottomleft = interpolate_linear_a(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_b(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let leftbottom = interpolate_linear_a(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_b(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topright = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BL,
        Edge {
            path: [Pt(bottomleft, 0.), Pt(0., leftbottom)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RB,
            },
        },
    );

    cell.edges.insert(
        EnterType::LT,
        Edge {
            path: [Pt(0., lefttop), Pt(topright, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BR,
            },
        },
    );

    cell.edges.insert(
        EnterType::RT,
        Edge {
            path: [Pt(1., righttop), Pt(bottomright, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TR,
            },
        },
    );
}

fn heptagon_bl(cell: &mut Cell, opt: &Settings) {
    let bottomleft = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let leftbottom = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topleft = interpolate_linear_a(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let topright = interpolate_linear_b(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_b(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_a(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::LB,
        Edge {
            path: [Pt(0., leftbottom), Pt(topleft, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BL,
            },
        },
    );

    cell.edges.insert(
        EnterType::TR,
        Edge {
            path: [Pt(topright, 1.), Pt(1., righttop)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LT,
            },
        },
    );

    cell.edges.insert(
        EnterType::RB,
        Edge {
            path: [Pt(1., rightbottom), Pt(bottomleft, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TL,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(0., 0.),
    //         Pt(0., leftbottom),
    //         Pt(topleft, 1.),
    //         Pt(topright, 1.),
    //         Pt(1., righttop),
    //         Pt(1., rightbottom),
    //         Pt(bottomleft, 0.),
    //     ]
    // );
}

fn heptagon_tl(cell: &mut Cell, opt: &Settings) {
    let bottomleft = interpolate_linear_a(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_b(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_ab(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topleft = interpolate_linear_ab(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_b(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_a(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BL,
        Edge {
            path: [Pt(bottomleft, 0.), Pt(0., lefttop)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RT,
            },
        },
    );

    cell.edges.insert(
        EnterType::TL,
        Edge {
            path: [Pt(topleft, 1.), Pt(1., righttop)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LT,
            },
        },
    );

    cell.edges.insert(
        EnterType::RB,
        Edge {
            path: [Pt(1., rightbottom), Pt(bottomright, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomleft, 0.),
    //         Pt(0., lefttop),
    //         Pt(0., 1.),
    //         Pt(topleft, 1.),
    //         Pt(1., righttop),
    //         Pt(1., rightbottom),
    //         Pt(bottomright, 0.),
    //     ]
    // );
}

fn heptagon_br(cell: &mut Cell, opt: &Settings) {
    let bottomright = interpolate_linear_ab(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let leftbottom = interpolate_linear_a(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_b(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topleft = interpolate_linear_a(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let topright = interpolate_linear_b(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_ab(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BR,
        Edge {
            path: [Pt(bottomright, 0.), Pt(0., leftbottom)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RB,
            },
        },
    );

    cell.edges.insert(
        EnterType::LT,
        Edge {
            path: [Pt(0., lefttop), Pt(topleft, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BL,
            },
        },
    );

    cell.edges.insert(
        EnterType::TR,
        Edge {
            path: [Pt(topright, 1.), Pt(1., rightbottom)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LB,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomright, 0.),
    //         Pt(0., leftbottom),
    //         Pt(0., lefttop),
    //         Pt(topleft, 1.),
    //         Pt(topright, 1.),
    //         Pt(1., rightbottom),
    //         Pt(1., 0.),
    //     ]
    // );
}

fn octagon(cell: &mut Cell, opt: &Settings) {
    let bottomleft = interpolate_linear_a(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let bottomright = interpolate_linear_b(cell.x0, cell.x1, opt.min_v, opt.max_v);
    let leftbottom = interpolate_linear_a(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let lefttop = interpolate_linear_b(cell.x0, cell.x3, opt.min_v, opt.max_v);
    let topleft = interpolate_linear_a(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let topright = interpolate_linear_b(cell.x3, cell.x2, opt.min_v, opt.max_v);
    let righttop = interpolate_linear_b(cell.x1, cell.x2, opt.min_v, opt.max_v);
    let rightbottom = interpolate_linear_a(cell.x1, cell.x2, opt.min_v, opt.max_v);

    cell.edges.insert(
        EnterType::BL,
        Edge {
            path: [Pt(bottomleft, 0.), Pt(0., leftbottom)],
            move_info: MoveInfo {
                x: -1,
                y: 0,
                enter: EnterType::RB,
            },
        },
    );

    cell.edges.insert(
        EnterType::LT,
        Edge {
            path: [Pt(0., lefttop), Pt(topleft, 1.)],
            move_info: MoveInfo {
                x: 0,
                y: 1,
                enter: EnterType::BL,
            },
        },
    );

    cell.edges.insert(
        EnterType::TR,
        Edge {
            path: [Pt(topright, 1.), Pt(1., righttop)],
            move_info: MoveInfo {
                x: 1,
                y: 0,
                enter: EnterType::LT,
            },
        },
    );

    cell.edges.insert(
        EnterType::RB,
        Edge {
            path: [Pt(1., rightbottom), Pt(bottomright, 0.)],
            move_info: MoveInfo {
                x: 0,
                y: -1,
                enter: EnterType::TR,
            },
        },
    );

    // cell.polygons.push(
    //     vec![
    //         Pt(bottomleft, 0.),
    //         Pt(0., leftbottom),
    //         Pt(0., lefttop),
    //         Pt(topleft, 1.),
    //         Pt(topright, 1.),
    //         Pt(1., righttop),
    //         Pt(1., rightbottom),
    //         Pt(bottomright, 0.),
    //     ]
    // );
}

pub(crate) fn prepare_cell(
    x: usize,
    y: usize,
    data: &BorrowedGrid<f64>,
    opt: &Settings,
) -> Result<Option<Cell>> {
    /*  compose the 4-trit corner representation */
    let mut cval: u8 = 0;
    let x3 = *data.get(&(x, y + 1)).unwrap_or(&f64::NAN);
    let x2 = *data.get(&(x + 1, y + 1)).unwrap_or(&f64::NAN);
    let x1 = *data.get(&(x + 1, y)).unwrap_or(&f64::NAN);
    let x0 = *data.get(&(x, y)).unwrap_or(&f64::NAN);

    if x0.is_nan() || x1.is_nan() || x2.is_nan() || x3.is_nan() {
        return Ok(None);
    }

    /*
     * Here we detect the type of the cell
     *
     * x3 ---- x2
     * |      |
     * |      |
     * x0 ---- x1
     *
     * with edge points
     *
     * x0 = (x,y),
     * x1 = (x + 1, y),
     * x2 = (x + 1, y + 1), and
     * x3 = (x, y + 1)
     *
     * and compute the polygon intersections with the edges
     * of the cell. Each edge value may be (i) below, (ii) within,
     * or (iii) above the values of the isoband limits. We
     * encode this property using 2 bits of information, where
     *
     * 00 ... below,
     * 01 ... within, and
     * 10 ... above
     *
     * Then we store the cells value as vector
     *
     * cval = (x0, x1, x2, x3)
     *
     * where x0 are the two least significant bits (0th, 1st),
     * x1 the 2nd and 3rd bit, and so on. This essentially
     * enables us to work with a single integer number
     */
    cval |= if x3 < opt.min_v {
        0
    } else if x3 > opt.max_v {
        128
    } else {
        64
    };
    cval |= if x2 < opt.min_v {
        0
    } else if x2 > opt.max_v {
        32
    } else {
        16
    };
    cval |= if x1 < opt.min_v {
        0
    } else if x1 > opt.max_v {
        8
    } else {
        4
    };
    cval |= if x0 < opt.min_v {
        0
    } else if x0 > opt.max_v {
        2
    } else {
        1
    };

    /*
     * cell center average trit for ambiguous cases, where
     * 0 ... below iso band
     * 1 ... within iso band
     * 2 ... above isoband
     */
    let mut center_avg: u8 = 0;

    let mut cell = Cell {
        // x,
        // y,
        // cval,
        x0,
        x1,
        x2,
        x3,
        edges: FxHashMap::default(),
    };

    // I tried storing the functions in a hashmap (FxHashMap) and in a Vec
    // but none of them were faster than the match statement
    // (which I decided to keep for now for readability).
    match cval {
        0 | 170 => {} /* 0000 or 2222 */
        85 => {
            /* 1111 */
            square(&mut cell, opt);
        }
        169 => {
            /* 2221 */
            triangle_bl(&mut cell, opt);
        }
        166 => {
            /* 2212 */
            triangle_br(&mut cell, opt);
        }
        154 => {
            /* 2122 */
            triangle_tr(&mut cell, opt);
        }
        106 => {
            /* 1222 */
            triangle_tl(&mut cell, opt);
        }
        1 => {
            /* 0001 */
            triangle_bl(&mut cell, opt);
        }
        4 => {
            /* 0010 */
            triangle_br(&mut cell, opt);
        }
        16 => {
            /* 0100 */
            triangle_tr(&mut cell, opt);
        }
        64 => {
            /* 1000 */
            triangle_tl(&mut cell, opt);
        }

        /* single trapezoid cases */
        168 => {
            /* 2220 */
            tetragon_bl(&mut cell, opt);
        }
        162 => {
            /* 2202 */
            tetragon_br(&mut cell, opt);
        }
        138 => {
            /* 2022 */
            tetragon_tr(&mut cell, opt);
        }
        42 => {
            /* 0222 */
            tetragon_tl(&mut cell, opt);
        }
        2 => {
            /* 0002 */
            tetragon_bl(&mut cell, opt);
        }
        8 => {
            /* 0020 */
            tetragon_br(&mut cell, opt);
        }
        32 => {
            /* 0200 */
            tetragon_tr(&mut cell, opt);
        }
        128 => {
            /* 2000 */
            tetragon_tl(&mut cell, opt);
        }

        /* single rectangle cases */
        5 => {
            /* 0011 */
            tetragon_b(&mut cell, opt);
        }
        20 => {
            /* 0110 */
            tetragon_r(&mut cell, opt);
        }
        80 => {
            /* 1100 */
            tetragon_t(&mut cell, opt);
        }
        65 => {
            /* 1001 */
            tetragon_l(&mut cell, opt);
        }
        165 => {
            /* 2211 */
            tetragon_b(&mut cell, opt);
        }
        150 => {
            /* 2112 */
            tetragon_r(&mut cell, opt);
        }
        90 => {
            /* 1122 */
            tetragon_t(&mut cell, opt);
        }
        105 => {
            /* 1221 */
            tetragon_l(&mut cell, opt);
        }
        160 => {
            /* 2200 */
            tetragon_lr(&mut cell, opt);
        }
        130 => {
            /* 2002 */
            tetragon_tb(&mut cell, opt);
        }
        10 => {
            /* 0022 */
            tetragon_lr(&mut cell, opt);
        }
        40 => {
            /* 0220 */
            tetragon_tb(&mut cell, opt);
        }

        /* single pentagon cases */
        101 => {
            /* 1211 */
            pentagon_tr(&mut cell, opt);
        }
        149 => {
            /* 2111 */
            pentagon_tl(&mut cell, opt);
        }
        86 => {
            /* 1112 */
            pentagon_bl(&mut cell, opt);
        }
        89 => {
            /* 1121 */
            pentagon_br(&mut cell, opt);
        }
        69 => {
            /* 1011 */
            pentagon_tr(&mut cell, opt);
        }
        21 => {
            /* 0111 */
            pentagon_tl(&mut cell, opt);
        }
        84 => {
            /* 1110 */
            pentagon_bl(&mut cell, opt);
        }
        81 => {
            /* 1101 */
            pentagon_br(&mut cell, opt);
        }
        96 => {
            /* 1200 */
            pentagon_tr_rl(&mut cell, opt);
        }
        24 => {
            /* 0120 */
            pentagon_rb_bt(&mut cell, opt);
        }
        6 => {
            /* 0012 */
            pentagon_bl_lr(&mut cell, opt);
        }
        129 => {
            /* 2001 */
            pentagon_lt_tb(&mut cell, opt);
        }
        74 => {
            /* 1022 */
            pentagon_tr_rl(&mut cell, opt);
        }
        146 => {
            /* 2102 */
            pentagon_rb_bt(&mut cell, opt);
        }
        164 => {
            /* 2210 */
            pentagon_bl_lr(&mut cell, opt);
        }
        41 => {
            /* 0221 */
            pentagon_lt_tb(&mut cell, opt);
        }
        66 => {
            /* 1002 */
            pentagon_bl_tb(&mut cell, opt);
        }
        144 => {
            /* 2100 */
            pentagon_lt_rl(&mut cell, opt);
        }
        36 => {
            /* 0210 */
            pentagon_tr_bt(&mut cell, opt);
        }
        9 => {
            /* 0021 */
            pentagon_rb_lr(&mut cell, opt);
        }
        104 => {
            /* 1220 */
            pentagon_bl_tb(&mut cell, opt);
        }
        26 => {
            /* 0122 */
            pentagon_lt_rl(&mut cell, opt);
        }
        134 => {
            /* 2012 */
            pentagon_tr_bt(&mut cell, opt);
        }
        161 => {
            /* 2201 */
            pentagon_rb_lr(&mut cell, opt);
        }

        /* single hexagon cases */
        37 => {
            /* 0211 */
            hexagon_lt_tr(&mut cell, opt);
        }
        148 => {
            /* 2110 */
            hexagon_bl_lt(&mut cell, opt);
        }
        82 => {
            /* 1102 */
            hexagon_bl_rb(&mut cell, opt);
        }
        73 => {
            /* 1021 */
            hexagon_tr_rb(&mut cell, opt);
        }
        133 => {
            /* 2011 */
            hexagon_lt_tr(&mut cell, opt);
        }
        22 => {
            /* 0112 */
            hexagon_bl_lt(&mut cell, opt);
        }
        88 => {
            /* 1120 */
            hexagon_bl_rb(&mut cell, opt);
        }
        97 => {
            /* 1201 */
            hexagon_tr_rb(&mut cell, opt);
        }
        145 => {
            /* 2101 */
            hexagon_lt_rb(&mut cell, opt);
        }
        25 => {
            /* 0121 */
            hexagon_lt_rb(&mut cell, opt);
        }
        70 => {
            /* 1012 */
            hexagon_bl_tr(&mut cell, opt);
        }
        100 => {
            /* 1210 */
            hexagon_bl_tr(&mut cell, opt);
        }
        17 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 2 */
            if center_avg == 0 {
                triangle_bl(&mut cell, opt);
                triangle_tr(&mut cell, opt);
            } else {
                hexagon_lt_rb(&mut cell, opt);
            }
        }
        68 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 2 */
            if center_avg == 0 {
                triangle_tl(&mut cell, opt);
                triangle_br(&mut cell, opt);
            } else {
                hexagon_bl_tr(&mut cell, opt);
            }
        }
        153 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 0 */
            if center_avg == 2 {
                triangle_bl(&mut cell, opt);
                triangle_tr(&mut cell, opt);
            } else {
                hexagon_lt_rb(&mut cell, opt);
            }
        }
        102 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 0 */
            if center_avg == 2 {
                triangle_tl(&mut cell, opt);
                triangle_br(&mut cell, opt);
            } else {
                hexagon_bl_tr(&mut cell, opt);
            }
        }
        152 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 0 */
            if center_avg == 2 {
                triangle_tr(&mut cell, opt);
                tetragon_bl(&mut cell, opt);
            } else {
                heptagon_tr(&mut cell, opt);
            }
        }
        137 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 0 */
            if center_avg == 2 {
                triangle_bl(&mut cell, opt);
                tetragon_tr(&mut cell, opt);
            } else {
                heptagon_bl(&mut cell, opt);
            }
        }
        98 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 0 */
            if center_avg == 2 {
                triangle_tl(&mut cell, opt);
                tetragon_br(&mut cell, opt);
            } else {
                heptagon_tl(&mut cell, opt);
            }
        }
        38 => {
            compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 0 */
            if center_avg == 2 {
                triangle_br(&mut cell, opt);
                tetragon_tl(&mut cell, opt);
            } else {
                heptagon_br(&mut cell, opt);
            }
        }
        18 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 2 */
            if center_avg == 0 {
                triangle_tr(&mut cell, opt);
                tetragon_bl(&mut cell, opt);
            } else {
                heptagon_tr(&mut cell, opt);
            }
        }
        33 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 2 */
            if center_avg == 0 {
                triangle_bl(&mut cell, opt);
                tetragon_tr(&mut cell, opt);
            } else {
                heptagon_bl(&mut cell, opt);
            }
        }
        72 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 2 */
            if center_avg == 0 {
                triangle_tl(&mut cell, opt);
                tetragon_br(&mut cell, opt);
            } else {
                heptagon_tl(&mut cell, opt);
            }
        }
        132 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            /* should never be center_avg === 2 */
            if center_avg == 0 {
                triangle_br(&mut cell, opt);
                tetragon_tl(&mut cell, opt);
            } else {
                heptagon_br(&mut cell, opt);
            }
        }
        136 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);
            if center_avg == 0 {
                tetragon_tl(&mut cell, opt);
                tetragon_br(&mut cell, opt);
            } else if center_avg == 1 {
                octagon(&mut cell, opt);
            } else {
                tetragon_bl(&mut cell, opt);
                tetragon_tr(&mut cell, opt);
            }
        }
        34 => {
            center_avg = compute_center_average(x0, x1, x2, x3, opt.min_v, opt.max_v);

            if center_avg == 0 {
                tetragon_bl(&mut cell, opt);
                tetragon_tr(&mut cell, opt);
            } else if center_avg == 1 {
                octagon(&mut cell, opt);
            } else {
                tetragon_tl(&mut cell, opt);
                tetragon_br(&mut cell, opt);
            }
        }
        _ => return Err(new_error(ErrorKind::UnexpectedCVAL)),
    };
    // Use a map instead of a match
    // if let Some(fn_case) = CVAL_MAP.get(&cval) {
    //     fn_case(&mut cell, opt);
    // } else {
    //     return Err(new_error(ErrorKind::UnexpectedCVAL));
    // }
    // Use a Vec instead of a map
    // if let Some(bucket_content) = CVAL_MAP.get(cval) {
    //     if let Some(fn_case) = bucket_content {
    //         fn_case(&mut cell, opt);
    //     } else {
    //         return Err(new_error(ErrorKind::UnexpectedCVAL));
    //     }
    // } else {
    //     return Err(new_error(ErrorKind::UnexpectedCVAL));
    // }

    Ok(Some(cell))
}

#[cfg(test)]
mod test {
    use crate::shape_coordinates::compute_center_average;

    #[test]
    fn test_compute_center_average() {
        assert_eq!(compute_center_average(0., 0., 0., 0., 0., 1.), 1);
        assert_eq!(compute_center_average(1., 1., 0., 0., 0., 0.), 2);
        assert_eq!(compute_center_average(1., 1., 0., 0., 0., 1.), 1);
    }
}

// Below are attempts to use a map or a vec instead of a match
// in the prepare_cell function.

// fn no_op(_cell: &mut Cell, _opt: &Settings) {
// }
//
// fn case17(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 2 */
//     if center_avg == 0 {
//         triangle_bl(cell, opt);
//         triangle_tr(cell, opt);
//     } else {
//         hexagon_lt_rb(cell, opt);
//     }
// }
//
// fn case68(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 2 */
//     if center_avg == 0 {
//         triangle_tl(cell, opt);
//         triangle_br(cell, opt);
//     } else {
//         hexagon_bl_tr(cell, opt);
//     }
// }
//
// fn case153(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 0 */
//     if center_avg == 2 {
//         triangle_bl(cell, opt);
//         triangle_tr(cell, opt);
//     } else {
//         hexagon_lt_rb(cell, opt);
//     }
// }
//
// fn case102(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 0 */
//     if center_avg == 2 {
//         triangle_tl(cell, opt);
//         triangle_br(cell, opt);
//     } else {
//         hexagon_bl_tr(cell, opt);
//     }
// }
//
// fn case152(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 0 */
//     if center_avg == 2 {
//         triangle_tr(cell, opt);
//         tetragon_bl(cell, opt);
//     } else {
//         heptagon_tr(cell, opt);
//     }
// }
//
// fn case137(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 0 */
//     if center_avg == 2 {
//         triangle_bl(cell, opt);
//         tetragon_tr(cell, opt);
//     } else {
//         heptagon_bl(cell, opt);
//     }
// }
//
// fn case98(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 0 */
//     if center_avg == 2 {
//         triangle_tl(cell, opt);
//         tetragon_br(cell, opt);
//     } else {
//         heptagon_tl(cell, opt);
//     }
// }
//
// fn case38(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 0 */
//     if center_avg == 2 {
//         triangle_br(cell, opt);
//         tetragon_tl(cell, opt);
//     } else {
//         heptagon_br(cell, opt);
//     }
// }
//
// fn case18(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 2 */
//     if center_avg == 0 {
//         triangle_tr(cell, opt);
//         tetragon_bl(cell, opt);
//     } else {
//         heptagon_tr(cell, opt);
//     }
// }
//
// fn case33(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 2 */
//     if center_avg == 0 {
//         triangle_bl(cell, opt);
//         tetragon_tr(cell, opt);
//     } else {
//         heptagon_bl(cell, opt);
//     }
// }
//
// fn case72(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 2 */
//     if center_avg == 0 {
//         triangle_tl(cell, opt);
//         tetragon_br(cell, opt);
//     } else {
//         heptagon_tl(cell, opt);
//     }
// }
//
// fn case132(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//     /* should never be center_avg === 2 */
//     if center_avg == 0 {
//         triangle_br(cell, opt);
//         tetragon_tl(cell, opt);
//     } else {
//         heptagon_br(cell, opt);
//     }
// }
//
// fn case136(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//
//     if center_avg == 0 {
//         tetragon_tl(cell, opt);
//         tetragon_br(cell, opt);
//     } else if center_avg == 1 {
//         octagon(cell, opt);
//     } else {
//         tetragon_bl(cell, opt);
//         tetragon_tr(cell, opt);
//     }
// }
//
// fn case34(cell: &mut Cell, opt: &Settings) {
//     let center_avg =
//         compute_center_average(cell.x0, cell.x1, cell.x2, cell.x3, opt.min_v, opt.max_v);
//
//     if center_avg == 0 {
//         tetragon_bl(cell, opt);
//         tetragon_tr(cell, opt);
//     } else if center_avg == 1 {
//         octagon(cell, opt);
//     } else {
//         tetragon_tl(cell, opt);
//         tetragon_br(cell, opt);
//     }
// }
//
// lazy_static! {
//     static ref CVAL_MAP: Vec<Option<fn(&mut Cell, &Settings)>> = {
//         let mut m = Vec::with_capacity(171);
//         for i in 0..171 {
//             m.push(None);
//         }
//         m[0] = Some(no_op as fn(&mut Cell, &Settings));
//         m[170] = Some(no_op as fn(&mut Cell, &Settings));
//         m[85] = Some(square as fn(&mut Cell, &Settings));
//         m[169] = Some(triangle_bl as fn(&mut Cell, &Settings));
//         m[166] = Some(triangle_br as fn(&mut Cell, &Settings));
//         m[154] = Some(triangle_tr as fn(&mut Cell, &Settings));
//         m[106] = Some(triangle_tl as fn(&mut Cell, &Settings));
//         m[1] = Some(triangle_bl as fn(&mut Cell, &Settings));
//         m[4] = Some(triangle_br as fn(&mut Cell, &Settings));
//         m[16] = Some(triangle_tr as fn(&mut Cell, &Settings));
//         m[64] = Some(triangle_tl as fn(&mut Cell, &Settings));
//         m[168] = Some(tetragon_bl as fn(&mut Cell, &Settings));
//         m[162] = Some(tetragon_br as fn(&mut Cell, &Settings));
//         m[138] = Some(tetragon_tr as fn(&mut Cell, &Settings));
//         m[42] = Some(tetragon_tl as fn(&mut Cell, &Settings));
//         m[2] = Some(tetragon_bl as fn(&mut Cell, &Settings));
//         m[8] = Some(tetragon_br as fn(&mut Cell, &Settings));
//         m[32] = Some(tetragon_tr as fn(&mut Cell, &Settings));
//         m[128] = Some(tetragon_tl as fn(&mut Cell, &Settings));
//         m[5] = Some(tetragon_b as fn(&mut Cell, &Settings));
//         m[20] = Some(tetragon_r as fn(&mut Cell, &Settings));
//         m[80] = Some(tetragon_t as fn(&mut Cell, &Settings));
//         m[65] = Some(tetragon_l as fn(&mut Cell, &Settings));
//         m[165] = Some(tetragon_b as fn(&mut Cell, &Settings));
//         m[150] = Some(tetragon_r as fn(&mut Cell, &Settings));
//         m[90] = Some(tetragon_t as fn(&mut Cell, &Settings));
//         m[105] = Some(tetragon_l as fn(&mut Cell, &Settings));
//         m[160] = Some(tetragon_lr as fn(&mut Cell, &Settings));
//         m[130] = Some(tetragon_tb as fn(&mut Cell, &Settings));
//         m[10] = Some(tetragon_lr as fn(&mut Cell, &Settings));
//         m[40] = Some(tetragon_tb as fn(&mut Cell, &Settings));
//         m[101] = Some(pentagon_tr as fn(&mut Cell, &Settings));
//         m[149] = Some(pentagon_tl as fn(&mut Cell, &Settings));
//         m[86] = Some(pentagon_bl as fn(&mut Cell, &Settings));
//         m[89] = Some(pentagon_br as fn(&mut Cell, &Settings));
//         m[69] = Some(pentagon_tr as fn(&mut Cell, &Settings));
//         m[21] = Some(pentagon_tl as fn(&mut Cell, &Settings));
//         m[84] = Some(pentagon_bl as fn(&mut Cell, &Settings));
//         m[81] = Some(pentagon_br as fn(&mut Cell, &Settings));
//         m[96] = Some(pentagon_tr_rl as fn(&mut Cell, &Settings));
//         m[24] = Some(pentagon_rb_bt as fn(&mut Cell, &Settings));
//         m[6] = Some(pentagon_bl_lr as fn(&mut Cell, &Settings));
//         m[129] = Some(pentagon_lt_tb as fn(&mut Cell, &Settings));
//         m[74] = Some(pentagon_tr_rl as fn(&mut Cell, &Settings));
//         m[146] = Some(pentagon_rb_bt as fn(&mut Cell, &Settings));
//         m[164] = Some(pentagon_bl_lr as fn(&mut Cell, &Settings));
//         m[41] = Some(pentagon_lt_tb as fn(&mut Cell, &Settings));
//         m[66] = Some(pentagon_bl_tb as fn(&mut Cell, &Settings));
//         m[144] = Some(pentagon_lt_rl as fn(&mut Cell, &Settings));
//         m[36] = Some(pentagon_tr_bt as fn(&mut Cell, &Settings));
//         m[9] = Some(pentagon_rb_lr as fn(&mut Cell, &Settings));
//         m[104] = Some(pentagon_bl_tb as fn(&mut Cell, &Settings));
//         m[26] = Some(pentagon_lt_rl as fn(&mut Cell, &Settings));
//         m[134] = Some(pentagon_tr_bt as fn(&mut Cell, &Settings));
//         m[161] = Some(pentagon_rb_lr as fn(&mut Cell, &Settings));
//         m[37] = Some(hexagon_lt_tr as fn(&mut Cell, &Settings));
//         m[148] = Some(hexagon_bl_lt as fn(&mut Cell, &Settings));
//         m[82] = Some(hexagon_bl_rb as fn(&mut Cell, &Settings));
//         m[73] = Some(hexagon_tr_rb as fn(&mut Cell, &Settings));
//         m[133] = Some(hexagon_lt_tr as fn(&mut Cell, &Settings));
//         m[22] = Some(hexagon_bl_lt as fn(&mut Cell, &Settings));
//         m[88] = Some(hexagon_bl_rb as fn(&mut Cell, &Settings));
//         m[97] = Some(hexagon_tr_rb as fn(&mut Cell, &Settings));
//         m[145] = Some(hexagon_lt_rb as fn(&mut Cell, &Settings));
//         m[25] = Some(hexagon_lt_rb as fn(&mut Cell, &Settings));
//         m[70] = Some(hexagon_bl_tr as fn(&mut Cell, &Settings));
//         m[100] = Some(hexagon_bl_tr as fn(&mut Cell, &Settings));
//         m[17] = Some(case17 as fn(&mut Cell, &Settings));
//         m[68] = Some(case68 as fn(&mut Cell, &Settings));
//         m[153] = Some(case153 as fn(&mut Cell, &Settings));
//         m[102] = Some(case102 as fn(&mut Cell, &Settings));
//         m[152] = Some(case152 as fn(&mut Cell, &Settings));
//         m[137] = Some(case137 as fn(&mut Cell, &Settings));
//         m[98] = Some(case98 as fn(&mut Cell, &Settings));
//         m[38] = Some(case38 as fn(&mut Cell, &Settings));
//         m[18] = Some(case18 as fn(&mut Cell, &Settings));
//         m[33] = Some(case33 as fn(&mut Cell, &Settings));
//         m[72] = Some(case72 as fn(&mut Cell, &Settings));
//         m[132] = Some(case132 as fn(&mut Cell, &Settings));
//         m[136] = Some(case136 as fn(&mut Cell, &Settings));
//         m[34] = Some(case34 as fn(&mut Cell, &Settings));
//         m
//     };
// }
// lazy_static! {
//     static ref CVAL_MAP: HashMap<u8, fn(&mut Cell, &Settings), BuildHasherDefault<FxHasher>> = {
//         let mut m = FxHashMap::default();
//         m.insert(0, no_op as fn(&mut Cell, &Settings));
//         m.insert(170, no_op as fn(&mut Cell, &Settings));
//         m.insert(85, square as fn(&mut Cell, &Settings));
//         m.insert(169, triangle_bl as fn(&mut Cell, &Settings));
//         m.insert(166, triangle_br as fn(&mut Cell, &Settings));
//         m.insert(154, triangle_tr as fn(&mut Cell, &Settings));
//         m.insert(106, triangle_tl as fn(&mut Cell, &Settings));
//         m.insert(1, triangle_bl as fn(&mut Cell, &Settings));
//         m.insert(4, triangle_br as fn(&mut Cell, &Settings));
//         m.insert(16, triangle_tr as fn(&mut Cell, &Settings));
//         m.insert(64, triangle_tl as fn(&mut Cell, &Settings));
//         m.insert(168, tetragon_bl as fn(&mut Cell, &Settings));
//         m.insert(162, tetragon_br as fn(&mut Cell, &Settings));
//         m.insert(138, tetragon_tr as fn(&mut Cell, &Settings));
//         m.insert(42, tetragon_tl as fn(&mut Cell, &Settings));
//         m.insert(2, tetragon_bl as fn(&mut Cell, &Settings));
//         m.insert(8, tetragon_br as fn(&mut Cell, &Settings));
//         m.insert(32, tetragon_tr as fn(&mut Cell, &Settings));
//         m.insert(128, tetragon_tl as fn(&mut Cell, &Settings));
//         m.insert(5, tetragon_b as fn(&mut Cell, &Settings));
//         m.insert(20, tetragon_r as fn(&mut Cell, &Settings));
//         m.insert(80, tetragon_t as fn(&mut Cell, &Settings));
//         m.insert(65, tetragon_l as fn(&mut Cell, &Settings));
//         m.insert(165, tetragon_b as fn(&mut Cell, &Settings));
//         m.insert(150, tetragon_r as fn(&mut Cell, &Settings));
//         m.insert(90, tetragon_t as fn(&mut Cell, &Settings));
//         m.insert(105, tetragon_l as fn(&mut Cell, &Settings));
//         m.insert(160, tetragon_lr as fn(&mut Cell, &Settings));
//         m.insert(130, tetragon_tb as fn(&mut Cell, &Settings));
//         m.insert(10, tetragon_lr as fn(&mut Cell, &Settings));
//         m.insert(40, tetragon_tb as fn(&mut Cell, &Settings));
//         m.insert(101, pentagon_tr as fn(&mut Cell, &Settings));
//         m.insert(149, pentagon_tl as fn(&mut Cell, &Settings));
//         m.insert(86, pentagon_bl as fn(&mut Cell, &Settings));
//         m.insert(89, pentagon_br as fn(&mut Cell, &Settings));
//         m.insert(69, pentagon_tr as fn(&mut Cell, &Settings));
//         m.insert(21, pentagon_tl as fn(&mut Cell, &Settings));
//         m.insert(84, pentagon_bl as fn(&mut Cell, &Settings));
//         m.insert(81, pentagon_br as fn(&mut Cell, &Settings));
//         m.insert(96, pentagon_tr_rl as fn(&mut Cell, &Settings));
//         m.insert(24, pentagon_rb_bt as fn(&mut Cell, &Settings));
//         m.insert(6, pentagon_bl_lr as fn(&mut Cell, &Settings));
//         m.insert(129, pentagon_lt_tb as fn(&mut Cell, &Settings));
//         m.insert(74, pentagon_tr_rl as fn(&mut Cell, &Settings));
//         m.insert(146, pentagon_rb_bt as fn(&mut Cell, &Settings));
//         m.insert(164, pentagon_bl_lr as fn(&mut Cell, &Settings));
//         m.insert(41, pentagon_lt_tb as fn(&mut Cell, &Settings));
//         m.insert(66, pentagon_bl_tb as fn(&mut Cell, &Settings));
//         m.insert(144, pentagon_lt_rl as fn(&mut Cell, &Settings));
//         m.insert(36, pentagon_tr_bt as fn(&mut Cell, &Settings));
//         m.insert(9, pentagon_rb_lr as fn(&mut Cell, &Settings));
//         m.insert(104, pentagon_bl_tb as fn(&mut Cell, &Settings));
//         m.insert(26, pentagon_lt_rl as fn(&mut Cell, &Settings));
//         m.insert(134, pentagon_tr_bt as fn(&mut Cell, &Settings));
//         m.insert(161, pentagon_rb_lr as fn(&mut Cell, &Settings));
//         m.insert(37, hexagon_lt_tr as fn(&mut Cell, &Settings));
//         m.insert(148, hexagon_bl_lt as fn(&mut Cell, &Settings));
//         m.insert(82, hexagon_bl_rb as fn(&mut Cell, &Settings));
//         m.insert(73, hexagon_tr_rb as fn(&mut Cell, &Settings));
//         m.insert(133, hexagon_lt_tr as fn(&mut Cell, &Settings));
//         m.insert(22, hexagon_bl_lt as fn(&mut Cell, &Settings));
//         m.insert(88, hexagon_bl_rb as fn(&mut Cell, &Settings));
//         m.insert(97, hexagon_tr_rb as fn(&mut Cell, &Settings));
//         m.insert(145, hexagon_lt_rb as fn(&mut Cell, &Settings));
//         m.insert(25, hexagon_lt_rb as fn(&mut Cell, &Settings));
//         m.insert(70, hexagon_bl_tr as fn(&mut Cell, &Settings));
//         m.insert(100, hexagon_bl_tr as fn(&mut Cell, &Settings));
//         m.insert(17, case17 as fn(&mut Cell, &Settings));
//         m.insert(68, case68 as fn(&mut Cell, &Settings));
//         m.insert(153, case153 as fn(&mut Cell, &Settings));
//         m.insert(102, case102 as fn(&mut Cell, &Settings));
//         m.insert(152, case152 as fn(&mut Cell, &Settings));
//         m.insert(137, case137 as fn(&mut Cell, &Settings));
//         m.insert(98, case98 as fn(&mut Cell, &Settings));
//         m.insert(38, case38 as fn(&mut Cell, &Settings));
//         m.insert(18, case18 as fn(&mut Cell, &Settings));
//         m.insert(33, case33 as fn(&mut Cell, &Settings));
//         m.insert(72, case72 as fn(&mut Cell, &Settings));
//         m.insert(132, case132 as fn(&mut Cell, &Settings));
//         m.insert(136, case136 as fn(&mut Cell, &Settings));
//         m.insert(34, case34 as fn(&mut Cell, &Settings));
//         m
//     };
// }
