use bevy::prelude::*;
use perlin_noise::PerlinNoise;
use crate::constants::MAX_TERRAIN_HEIGHT;
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

        dbg!(rat_w, rat_h);
        let mut noise = PerlinNoise::new();

        let mut hm = HeightMap {
            w,
            h,
            cell_w,
            cell_h,
            rat_w,
            rat_h,
            map,
        };
        dbg!("hm dims:", hm.map.len(), w, h, cell_w, cell_h);

        hm.terraform(&mut noise);
        hm
    }

    pub fn terraform(&mut self, noise: &mut PerlinNoise) {
        let size = 0.05;
        let terrain_height = MAX_TERRAIN_HEIGHT;// * ratio;

        for y in 0..self.cell_h {
            for x in 0..self.cell_w {
                let mut h = noise.get3d([
                    x as f64 * size,
                    y as f64 * size,
                    0.0,
                ]);
                //println!("{} {} = {} {}", xo, ((x as i32 + xo) as f64) * size, yo, ((y as i32 + yo) as f64) * size);
                let h2 = noise.get3d([
                    (x as f64 * size * 20.0),
                    (y as f64 * size * 20.0),
                    0.0,
                ]) * 0.05;
                h += h2;
                h = h.max(0.5) - 0.5;

                // Make "halfpipe"
                //let pp = 1.0 - ((x as f32 / map.cell_w as f32) * 3.1415).sin();
                let px =  ((x as f32 / self.cell_w as f32) - 0.5) * 2.0;
                let pp = px.powf(12.0);

                let ratio = y as f32 / self.cell_h as f32;
                let height = h as f32 * terrain_height + (pp * 50.0);
                self.map[y][x] = height * ratio;
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
        if cell_x >= self.cell_w || cell_y >= self.cell_h {
            dbg!(cell_x, self.cell_w, cell_y, self.cell_h);
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
        let mut rng = rand::thread_rng();
        let cell_x = rng.gen_range(0..self.cell_w);
        let cell_y = rng.gen_range(0..self.cell_h);
        (cell_x, cell_y)
    }
}
