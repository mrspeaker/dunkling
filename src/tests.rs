#[cfg(test)]
mod tests {
    // use super::*;
    use crate::height_map::HeightMap;
    use crate::sheet::get_neighbours_radius;
    
    #[test]
    fn pos_to_cell() {
        let mut height_map = HeightMap::new(100.0, 100.0, 10, 10);

        let cell = height_map.get_cell_from_pos(0.0, 0.0);
        assert_eq!(cell, Some((0, 0)));

        let cell = height_map.get_cell_from_pos(50.0, 50.0);
        assert_eq!(cell, Some((5, 5)));

        height_map.map[0][0] = 0.5;
        let h = height_map.pos_to_height(0.0, 0.0);
        assert_eq!(h, Some(0.5));

        height_map.map[1][1] = 2.0;                
        let h = height_map.pos_to_height(10.0, 10.0);
        assert_eq!(h, Some(2.0));

        height_map.map[2][9] = 2.5;                
        let h = height_map.pos_to_height(95.0, 25.0);
        assert_eq!(h, Some(2.5));

        let h = height_map.pos_to_height(100.1, 0.0);
        assert_eq!(h, None);

        // Should fail I guess...
        //let h = height_map.pos_to_height(-0.1, 0.0);
        //assert_eq!(h, None);
    }

    #[test]
    fn get_neighbours() {
        let n = get_neighbours_radius(2, 2, 1);
        assert_eq!(n.len(), 9);
        assert_eq!(n[0], (1, 1, 0.0));
        assert_eq!(n[1], (2, 1, 0.29289323));
        assert_eq!(n[4], (2, 2, 1.0));
        assert_eq!(n[8], (3, 3, 0.0));
    }

    #[test]
    fn get_neighbours_sat() {
        let n = get_neighbours_radius(0, 0, 1);
        assert_eq!(n.len(), 4);
        assert_eq!(n[0], (0, 0, 1.0));
        assert_eq!(n[3], (1, 1, 0.0));
    }
    #[test]
    fn get_neighbours_wide() {
        let n = get_neighbours_radius(0, 0, 2);
        assert_eq!(n.len(), 9);
        assert_eq!(n[8], (2, 2, 0.0));
    }
    #[test]
    fn get_neighbours_vwide() {
        let n = get_neighbours_radius(0, 0, 3);
        assert_eq!(n.len(), 16);
        assert_eq!(n[15], (3, 3, 0.0));
    }
}

