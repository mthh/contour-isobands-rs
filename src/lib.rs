mod errors;
mod isobands;
mod polygons;
mod shape_coordinates;

pub use crate::isobands::isobands;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isobands::Pt;

    #[test]
    fn isoband_simple() {
        let matrix = vec![
            vec![1., 1., 1., 0.],
            vec![1., 5., 5., 1.],
            vec![0., 1., 1., 1.],
        ];
        let lower_band = 1.;
        let bandwidth = 1.;

        let res = isobands(matrix, vec![lower_band], vec![bandwidth]);
        assert_eq!(
            res,
            vec![vec![
                vec![
                    Pt(0.25, 1.),
                    Pt(1., 0.25),
                    Pt(2., 0.25),
                    Pt(2.75, 1.),
                    Pt(2., 1.75),
                    Pt(1., 1.75),
                    Pt(0.25, 1.),
                ],
                vec![
                    Pt(0., 1.),
                    Pt(1., 2.),
                    Pt(1., 2.),
                    Pt(2., 2.),
                    Pt(3., 2.),
                    Pt(3., 1.),
                    Pt(3., 1.),
                    Pt(2., 0.),
                    Pt(2., 0.),
                    Pt(1., 0.),
                    Pt(0., 0.),
                    Pt(0., 1.),
                ]
            ]]
        );

        println!("{:?}", res);
    }
}
