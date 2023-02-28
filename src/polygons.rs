use crate::errors::{new_error, ErrorKind, Result};
use crate::isobands::{Cell, EnterType, Pt, Settings};

fn require_frame(data: &[Vec<f64>], lowerbound: f64, upperbound: f64) -> bool {
    let mut frame_required: bool = true;
    let rows = data.len();
    let cols = data[0].len();

    for row in data.iter() {
        if row[0] < lowerbound
            || row[0] > upperbound
            || row[cols - 1] < lowerbound
            || row[cols - 1] > upperbound
        {
            frame_required = false;
            break;
        }
    }

    if frame_required && data[rows - 1][0] < lowerbound
        || data[rows - 1][0] > upperbound
        || data[rows - 1][cols - 1] < lowerbound
        || data[rows - 1][cols - 1] > upperbound
    {
        frame_required = false;
    }

    if frame_required {
        for i in 0..cols {
            if data[0][i] < lowerbound
                || data[0][i] > upperbound
                || data[rows - 1][i] < lowerbound
                || data[rows - 1][i] > upperbound
            {
                frame_required = false;
                break;
            }
        }
    }
    frame_required
}

fn entry_coordinate(x: i32, y: i32, mode: usize, path: &[Pt]) -> Pt {
    let mut x = x as f64;
    let mut y = y as f64;
    if mode == 0 {
        /* down */
        x += 1.;
        y += path[0].1;
    } else if mode == 1 {
        /* left */
        x += path[0].0;
    } else if mode == 2 {
        /* up */
        y += path[0].1;
    } else if mode == 3 {
        /* right */
        x += path[0].0;
        y += 1.;
    }

    Pt(x, y)
}

fn skip_coordinate(x: i32, y: i32, mode: usize) -> Pt {
    let mut x = x;
    let mut y = y;
    if mode == 0 {
        /* down */
        x += 1;
    } else if mode == 1 { /* left */
        /* Do nothing */
    } else if mode == 2 {
        /* up */
        y += 1;
    } else if mode == 3 {
        /* right */
        x += 1;
        y += 1;
    }

    Pt(x as f64, y as f64)
}

pub(crate) fn trace_band_paths(
    data: &[Vec<f64>],
    cell_grid: &mut Vec<Vec<Option<Cell>>>,
    opt: &Settings,
) -> Result<Vec<Vec<Pt>>> {
    let mut polygons: Vec<Vec<Pt>> = Vec::new();
    let rows = data.len() - 1;
    let cols = data[0].len() - 1;

    let available_starts = [
        EnterType::BL,
        EnterType::LB,
        EnterType::LT,
        EnterType::TL,
        EnterType::TR,
        EnterType::RT,
        EnterType::RB,
        EnterType::BR,
    ];

    let add_x = [0, -1, 0, 1];
    let add_y = [-1, 0, 1, 0];

    let entry_dir = |e: &EnterType| match e {
        EnterType::RT | EnterType::RB => 0,
        EnterType::BL | EnterType::BR => 1,
        EnterType::LB | EnterType::LT => 2,
        EnterType::TL | EnterType::TR => 3,
    };

    let valid_entries = [
        [EnterType::RT, EnterType::RB], /* down */
        [EnterType::BR, EnterType::BL], /* left */
        [EnterType::LB, EnterType::LT], /* up */
        [EnterType::TL, EnterType::TR], /* right */
    ];

    if require_frame(data, opt.min_v, opt.max_v) {
        polygons.push(vec![
            Pt(0., 0.),
            Pt(0., rows as f64),
            Pt(cols as f64, rows as f64),
            Pt(cols as f64, 0.),
            Pt(0., 0.),
        ]);
    }

    for i in 0..cell_grid.len() {
        for j in 0..cell_grid[i].len() {
            for nextedge in &available_starts {
                // println!("nextedge: {:?}", nextedge);
                if let Some(cg) = &cell_grid[i][j] {
                    if let Some(edge) = cg.edges.get(nextedge) {
                        let mut path = Vec::new();
                        let mut enter = nextedge.clone();

                        let mut x = i as i32;
                        let mut y = j as i32;
                        let mut finalized = false;
                        let origin = Pt(i as f64 + edge.path[0].0, j as f64 + edge.path[0].1);

                        path.push(origin.clone());

                        /* start traceback */
                        while !finalized {
                            if usize::try_from(x).is_err()
                                || usize::try_from(y).is_err()
                                || cell_grid.get(x as usize).is_none()
                                || cell_grid[x as usize].get(y as usize).is_none()
                            {
                                return Err(new_error(ErrorKind::OutOfBounds));
                            }

                            let mut _cc = cell_grid[x as usize][y as usize].as_mut();
                            // println!("x: {}, y: {}, enter: {:?}, cc: {:?}", x, y, enter, _cc);
                            if _cc.is_none() {
                                break;
                            }
                            let mut cc = _cc.unwrap();
                            /* remove edge from cell */
                            let mut _ee = cc.edges.get_remove(&enter);
                            if _ee.is_none() {
                                break;
                            }
                            let mut ee = &_ee.unwrap();

                            /* add last point of edge to path array, since we extend a polygon */
                            let point = Pt(ee.path[1].0 + x as f64, ee.path[1].1 + y as f64);
                            path.push(point);

                            enter = ee.move_info.enter.clone();
                            x += ee.move_info.x;
                            y += ee.move_info.y;

                            /* handle out-of-grid moves */
                            if usize::try_from(x).is_err()
                                || usize::try_from(y).is_err()
                                || cell_grid.get(x as usize).is_none()
                                || cell_grid[x as usize].get(y as usize).is_none()
                            {
                                let mut dir;
                                let mut count = 0;

                                if x == cols as i32 {
                                    x -= 1;
                                    dir = 0; /* move downwards */
                                } else if x < 0 {
                                    x += 1;
                                    dir = 2; /* move upwards */
                                } else if y == rows as i32 {
                                    y -= 1;
                                    dir = 3; /* move right */
                                } else if y < 0 {
                                    y += 1;
                                    dir = 1; /* move left */
                                } else {
                                    return Err(new_error(ErrorKind::UnexpectedOutOfGridMove));
                                }

                                if x == i as i32 && y == j as i32 && dir == entry_dir(nextedge) {
                                    // finalized = true;
                                    // enter = nextedge.clone();
                                    break;
                                }

                                loop {
                                    // println!("loop - dir = {}", dir);
                                    let mut found_entry = false;
                                    if count > 4 {
                                        println!("Direction change counter overflow! This should never happen!");
                                        break;
                                    }

                                    if usize::try_from(x).is_ok()
                                        && usize::try_from(y).is_ok()
                                        && cell_grid.get(x as usize).is_some()
                                        && cell_grid[x as usize].get(y as usize).is_some()
                                    // && cell_grid[x as usize][y as usize].is_some()
                                    {
                                        cc = cell_grid[x as usize][y as usize].as_mut().unwrap();

                                        /* check for re-entry */
                                        for s in 0..valid_entries[dir].len() {
                                            let ve = &valid_entries[dir][s];
                                            if cc.edges.get(ve).is_some() {
                                                /* found re-entry */
                                                ee = cc.edges.get(ve).unwrap();
                                                path.push(entry_coordinate(x, y, dir, &ee.path));
                                                enter = ve.clone();
                                                found_entry = true;
                                                break;
                                            }
                                        }
                                    }

                                    if found_entry {
                                        break;
                                    } else {
                                        path.push(skip_coordinate(x, y, dir));
                                        x += add_x[dir];
                                        y += add_y[dir];

                                        /* change direction if we moved out of grid again */
                                        if (usize::try_from(x).is_err()
                                            || usize::try_from(y).is_err()
                                            || cell_grid.get(x as usize).is_none()
                                            || cell_grid[x as usize].get(y as usize).is_none())
                                            && (((dir == 0) && (y < 0))
                                                || ((dir == 1) && (x < 0))
                                                || ((dir == 2) && (y == rows as i32))
                                                || ((dir == 3) && (x == cols as i32)))
                                        {
                                            x -= add_x[dir];
                                            y -= add_y[dir];

                                            dir = (dir + 1) % 4;
                                            count += 1;
                                        }

                                        if x == i as i32
                                            && y == j as i32
                                            && dir == entry_dir(nextedge)
                                        {
                                            finalized = true;
                                            enter = nextedge.clone();
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if path[path.len() - 1].0 != origin.0 || path[path.len() - 1].1 != origin.1
                        {
                            path.push(origin);
                        }

                        polygons.push(path);
                    }
                }
            }
        }
    }

    Ok(polygons)
}
