#[cfg(test)]
mod tests {
    // use super::*;
    use crate::sheet::HeightMap;
    
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
}
