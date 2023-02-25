use crate::isobands::{Cell, Corner, EnterType, Pt, Settings};
use std::str::FromStr;

fn require_frame(data: &Vec<Vec<f64>>, lowerbound: f64, upperbound: f64) -> bool {
    let mut frame_required: bool = true;
    let rows = data.len();
    let cols = data[0].len();

    for j in 0..rows {
        if data[j][0] < lowerbound
            || data[j][0] > upperbound
            || data[j][cols - 1] < lowerbound
            || data[j][cols - 1] > upperbound
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
    data: &Vec<Vec<f64>>,
    cell_grid: &mut Vec<Vec<Option<Cell>>>,
    opt: &Settings,
) -> Vec<Vec<Pt>> {
    let mut polygons: Vec<Vec<Pt>> = Vec::new();
    let rows = data.len() - 1;
    let cols = data[0].len() - 1;

    let available_starts = vec!["BL", "LB", "LT", "TL", "TR", "RT", "RB", "BR"];

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
            // let mut next_edge;
            for nextedge in &available_starts {
                println!("nextedge: {}", nextedge);
                let start_type = EnterType::from_str(nextedge).unwrap();
                if let Some(cd) = &cell_grid[i][j] {
                    if let Some(edge) = cd.edges.get(&start_type) {
                        let mut path = Vec::new();
                        let mut enter = EnterType::from_str(nextedge).unwrap();

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
                                panic!("Out of bounds");
                            }

                            let mut _cc = cell_grid[x as usize][y as usize].as_mut();
                            println!("x: {}, y: {}, enter: {:?}, cc: {:?}", x, y, enter, _cc);
                            if _cc.is_none() {
                                break;
                            }
                            let mut cc = _cc.unwrap();
                            /* remove edge from cell */
                            let mut _ee = cc.edges.get_remove(&enter);
                            if _ee.is_none() {
                                break;
                            }
                            let mut ee = _ee.unwrap();

                            /* add last point of edge to path array, since we extend a polygon */
                            let point = Pt(ee.path[1].0 + x as f64, ee.path[1].1 + y as f64);
                            path.push(point);

                            enter = ee.move_info.enter;
                            x = x + ee.move_info.x;
                            y = y + ee.move_info.y;

                            /* handle out-of-grid moves */
                            if usize::try_from(x).is_err()
                                || usize::try_from(y).is_err()
                                || cell_grid.get(x as usize).is_none()
                                || cell_grid[x as usize].get(y as usize).is_none()
                            {
                                let mut dir = 0;
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
                                    panic!("Unexpected out-of-grid move");
                                }

                                if x == i as i32 && y == j as i32 && dir == entry_dir(&start_type) {
                                    finalized = true;
                                    enter = start_type.clone();
                                    break;
                                }

                                loop {
                                    println!("foo");
                                    let mut found_entry = false;
                                    if count > 4 {
                                        println!("Direction change counter overflow! This should never happen!");
                                        break;
                                    }

                                    if !usize::try_from(x).is_err()
                                        && !usize::try_from(y).is_err()
                                        && !cell_grid.get(x as usize).is_none()
                                        && !cell_grid[x as usize].get(y as usize).is_none()
                                    // && !cell_grid[x as usize][y as usize].is_none()
                                    {
                                        cc = cell_grid[x as usize][y as usize].as_mut().unwrap();

                                        /* check for re-entry */
                                        for s in 0..valid_entries[dir].len() {
                                            let ve = valid_entries[dir][s].clone();
                                            if cc.edges.get(&ve).is_some() {
                                                /* found re-entry */
                                                ee = cc.edges.get(&ve).unwrap().clone();
                                                path.push(entry_coordinate(
                                                    x as i32, y as i32, dir, &ee.path,
                                                ));
                                                enter = ve;
                                                found_entry = true;
                                                break;
                                            }
                                        }
                                    }

                                    if found_entry {
                                        break;
                                    } else {
                                        path.push(skip_coordinate(x as i32, y as i32, dir));
                                        x += add_x[dir];
                                        y += add_y[dir];

                                        /* change direction if we'e moved out of grid again */
                                        if usize::try_from(x).is_err()
                                            || usize::try_from(y).is_err()
                                            || cell_grid.get(x as usize).is_none()
                                            || cell_grid[x as usize].get(y as usize).is_none()
                                        {
                                            if ((dir == 0) && (y < 0))
                                                || ((dir == 1) && (x < 0))
                                                || ((dir == 2) && (y == rows as i32))
                                                || ((dir == 3) && (x == cols as i32))
                                            {
                                                x -= add_x[dir];
                                                y -= add_y[dir];

                                                dir = (dir + 1) % 4;
                                                count += 1;
                                            }
                                        }

                                        if x == i as i32
                                            && y == j as i32
                                            && dir == entry_dir(&start_type)
                                        {
                                            finalized = true;
                                            enter = start_type.clone();
                                            break;
                                        }
                                    }
                                    // count += 1;
                                }
                            }
                        }

                        if path[path.len() - 1].0 != origin.0 || path[path.len() - 1].1 != origin.1
                        {
                            path.push(origin.clone());
                        }

                        polygons.push(path);
                    }
                }
            }
        }
    }

    polygons
}
