use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use crate::constants::{MAX_TERRAIN_HEIGHT, CELL_SIZE};
use rand::prelude::*;

#[derive(Resource, Clone, Debug)]
pub struct HeightMap {
    pub w: f32,
    pub h: f32,
    pub cell_w: usize,
    pub cell_h: usize,
    rat_w: f32,
    rat_h: f32,
    pub map: Vec<Vec<f32>>,
}

impl HeightMap {
    pub fn new(w: f32, h: f32, cell_w: usize, cell_h: usize) -> Self {
        let rat_w = w / cell_w as f32;
        let rat_h = h / cell_h as f32;
        let map = vec![vec![0.0; cell_w]; cell_h];

        let mut hm = HeightMap {
            w,
            h,
            cell_w,
            cell_h,
            rat_w,
            rat_h,
            map,
        };
        hm.terraform();
        hm
    }

    pub fn terraform(&mut self) {
        let mut rng = rand::rng();
        let noise = Perlin::new(rng.next_u32());
        let main_size = 0.01;
        let bump_size = 0.2;

        for y in 0..self.cell_h {
            for x in 0..self.cell_w {
                let noise_val  = noise.get([
                    x as f64 * main_size,
                    y as f64 * main_size,
                    0.0,
                ]);
                let bump_val  = noise.get([
                    x as f64 * bump_size,
                    y as f64 * bump_size,
                    0.0,
                ]) * 0.5;

                let noise_height = noise_val.max(0.0) as f32 * MAX_TERRAIN_HEIGHT;
                let bump_height = (bump_val * 2.5) as f32;

                // Make "halfpipe"
                let px =  ((x as f32 / self.cell_w as f32) - 0.5) * 2.0;
                let halfpipe = px.powf(12.0) * 50.0;

                let height = noise_height + halfpipe;

                let bump_ratio = (1.0 - (height / (MAX_TERRAIN_HEIGHT * 0.2))).max(0.0);

                // Increase slope along sheet z.
                // - Starts flat, goes bumpy, ends flat.
                // Curve: 1 - ((x / 0.4) - 1.25) ^ 4
                let z_percent = y as f32 / self.cell_h as f32;
                let curve = 1.0 - ((z_percent / 0.48) - 1.0).powf(4.0);
                let slope = curve.max(0.0); // Clip floor

                self.map[y][x] = ((height * slope) + (bump_height * bump_ratio)).max(0.0);
            }
        }

    }

    /// Given a SHEET x and y coordinate,
    /// return the corresponding CELL position.
    pub fn get_cell_from_pos(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        //Calculate the cell coordinates
        let cell_x = (x / self.rat_w).floor() as usize;
        let cell_y = (y / self.rat_h).floor() as usize;

        // Check if cell position is out of map bounds
        if cell_x >= self.cell_w || cell_y >= self.cell_h ||
        x < 0.0 || y < 0.0 {
            //info!(cell_x, self.cell_w, cell_y, self.cell_h);
            None // out of bound
        } else {
            Some((cell_x, cell_y))
        }
    }
    pub fn pos_to_height(&self, x:f32, y:f32) -> Option<f32> {
        let cell_pos = self.get_cell_from_pos(x, y);
        match cell_pos {
            Some((x, y)) => Some(self.map[y][x]),
            _ => None
        }
    }

    // Return a random cell x/y from the height map
    pub fn get_random_cell(&self) -> (usize, usize) {
        let mut rng = rand::rng();
        let cell_x = rng.random_range(0..self.cell_w);
        let cell_y = rng.random_range(0..self.cell_h);
        (cell_x, cell_y)
    }

    pub fn get_random_pos_between_height(&self, min_h: f32, max_h: f32) -> (f32, f32) {
        let mut i = 0;
        loop {
            let cell = self.get_random_cell();
            let h = self.map[cell.1][cell.0];
            if h >= min_h && h <= max_h {
                return (cell.0 as f32 * self.rat_w, cell.1 as f32 * self.rat_h)
            }
            i += 1;
            if i > 100 {
                return (0.0, 0.0);
            }
        }
    }


    pub fn add_height(&mut self, hm_x: usize, hm_y: usize, value: f32, chunk_idx: usize) {
        if hm_x >= self.cell_w ||
            hm_y >= self.cell_h {
                return;
            }

        //let map = &mut self.map;
        let zoff = hm_y + (chunk_idx * CELL_SIZE);
        let cur = (*self.map)[zoff][hm_x];
        let next = (cur + value).max(0.0);
        (*self.map)[zoff][hm_x] = next;
    }

}
