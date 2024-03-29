extern crate base64;
extern crate float_cmp;
extern crate rand;

pub mod block_address;

use float_cmp::*;
use macroquad::prelude::*;
use more_asserts::*;
use block_address::*;
//use rand::{rngs::StdRng, RngCore, SeedableRng};


/*
View coordinates are:
 block address

 position in block as Vec2 in range (0.0, 0.0) - (1.0, 1.0)
    center of the screen in world coordinates

 zoom_level - value from 0.0 to 1.0
    at zoom level 0.0 - block's width equals to screen width
     at zoom level 1.0 - one subblock's width equals to screen width
*/
#[derive(Debug, Clone)]
struct ViewPosition {
    block: BlockAddress,
    position: Vec2,
    zoom_level: f32,
}

impl PartialEq for ViewPosition {
    fn eq(&self, other: &Self) -> bool {
        return self.block.eq(&other.block)
            && self.position.abs_diff_eq(other.position, 1e-5)
            && self
                .zoom_level
                .approx_eq(other.zoom_level, F32Margin::default());
    }
}

impl ViewPosition {
    pub fn offset(&mut self, x: f32, y: f32) {
        let x = self.position.x + x;
        let y = self.position.y + y;

        let ix = x.floor();
        let iy = y.floor();
        self.position.x = x - ix;
        self.position.y = y - iy;

        assert_ge!(self.position.x, 0.0);
        assert_le!(self.position.x, 1.0);
        assert_ge!(self.position.y, 0.0);
        assert_le!(self.position.y, 1.0);

        self.block.offset(ix as i32, iy as i32);
    }

    pub fn zoom(&mut self, diff: f32) {
        let zoom = (self.block.get_zoom() as f32 + self.zoom_level + diff).max(0.0);

        let izoom = zoom.floor();
        self.zoom_level = zoom - izoom;
        let izoom = izoom as i32;

        while self.block.get_zoom() > izoom {
            let (block_x, block_y) = self.block.get_last_block_pos();
            self.block.zoom_out();
            self.position.x = (block_x as f32 + self.position.x) / SUBBLOCK_COUNT_F;
            self.position.y = (block_y as f32 + self.position.y) / SUBBLOCK_COUNT_F;

            assert_ge!(self.position.x, 0.0);
            assert_le!(self.position.x, 1.0);
            assert_ge!(self.position.y, 0.0);
            assert_le!(self.position.y, 1.0);
        }

        while self.block.get_zoom() < izoom {
            let block_x = (self.position.x * SUBBLOCK_COUNT_F).floor();
            let block_y = (self.position.y * SUBBLOCK_COUNT_F).floor();

            self.block.zoom_in(block_x as i32, block_y as i32);

            self.position.x = (self.position.x - block_x / SUBBLOCK_COUNT_F) * SUBBLOCK_COUNT_F;
            self.position.y = (self.position.y - block_y / SUBBLOCK_COUNT_F) * SUBBLOCK_COUNT_F;

            assert_ge!(self.position.x, 0.0);
            assert_le!(self.position.x, 1.0);
            assert_ge!(self.position.y, 0.0);
            assert_le!(self.position.y, 1.0);
        }
    }
}

fn draw_block(mat: &Mat3, block: &BlockAddress, color: Color, subcolors: &[Color]) {
    if let Some(&color) = subcolors.first() {
        let subcolors = &subcolors[1..];
        for i in 0..SUBBLOCK_COUNT {
            for j in 0..SUBBLOCK_COUNT {
                let mat = mat
                    .mul_mat3(&Mat3::from_scale(Vec2::new(
                        1.0 / SUBBLOCK_COUNT_F,
                        1.0 / SUBBLOCK_COUNT_F,
                    )))
                    .mul_mat3(&Mat3::from_translation(Vec2::new(i as f32, j as f32)));

                let mut block = block.clone();
                block.zoom_in(i, j);
                draw_block(&mat, &block, color, &subcolors);
            }
        }
    }

    let p00 = mat.transform_point2(Vec2::new(0.0, 0.0));
    let p10 = mat.transform_point2(Vec2::new(1.0, 0.0));
    let p01 = mat.transform_point2(Vec2::new(0.0, 1.0));
    let p11 = mat.transform_point2(Vec2::new(1.0, 1.0));

    let text_dim = measure_text(&block.get_name(), None, 20, 1.0);
    if text_dim.width < p10.x - p00.x {
        draw_text(
            &block.get_name(),
            p00.x + (p10.x - p00.x - text_dim.width) * 0.5,
            p00.y + (p01.y - p00.y - text_dim.height) * 0.5,
            20.0,
            WHITE,
        );
    }

    draw_line(p00.x, p00.y, p10.x, p10.y, 1.0, color);
    draw_line(p10.x, p10.y, p11.x, p11.y, 1.0, color);
    draw_line(p11.x, p11.y, p01.x, p01.y, 1.0, color);
    draw_line(p01.x, p01.y, p00.x, p00.y, 1.0, color);
}

fn lerp_colors(color0: Color, color1: Color, ratio: f32) -> Color {
    Color::from_vec(color0.to_vec().lerp(color1.to_vec(), ratio))
}

#[macroquad::main("ProceduralGalaxy")]
async fn main() {
    let mut position = ViewPosition {
        block: BlockAddress(vec![]),
        position: Vec2::new(0.5, 0.5),
        zoom_level: 0.0,
    };

    let mut time = get_time() as f32;

    loop {
        let dt = get_time() as f32 - time;
        time += dt;

        clear_background(RED);

        let mut dpos = Vec2::new(0.0, 0.0);
        let mut dzoom: f32 = 0.0;

        if is_key_down(KeyCode::Left) {
            dpos.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) {
            dpos.x += 1.0;
        }
        if is_key_down(KeyCode::Up) {
            dpos.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) {
            dpos.y += 1.0;
        }
        if is_key_down(KeyCode::Z) {
            dzoom -= 1.0;
        }
        if is_key_down(KeyCode::X) {
            dzoom += 1.0;
        }

        /*
         in world position.block is located in rect (0.0, 0.0, 1.0, 1.0)
         camera center is at position.position
         scale is such that
              at position.zoom_level == 0.0 - world's 1.0 is equal to screen_width
              at position.zoom_level == 1.0 - world's (1.0 / SUBBLOCK_COUNT) is equal to screen_width
        */

        let sw = screen_width() as f32;
        let sh = screen_height() as f32;
        let scale_0 = sw;
        let scale_1 = sw * SUBBLOCK_COUNT_F;
        let scale = scale_0 + position.zoom_level * (scale_1 - scale_0);

        let mat = Mat3::from_translation(Vec2::new(sw * 0.5, sh * 0.5))
            .mul_mat3(&Mat3::from_scale(Vec2::new(scale, scale)))
            .mul_mat3(&Mat3::from_translation(-position.position));

        let mat_inv = mat.inverse();
        let dpos = mat_inv.transform_vector2(dpos * sw * dt);
        position.offset(dpos.x, dpos.y);
        position.zoom(dzoom * dt);

        // camera rect in world coordinates
        let s00 = mat_inv.transform_point2(Vec2::new(0.0, 0.0));
        let s11 = mat_inv.transform_point2(Vec2::new(sw, sh));

        for x in s00.x.floor() as i32..s11.x.ceil() as i32 {
            for y in s00.y.floor() as i32..s11.y.ceil() as i32 {
                let mut block = position.block.clone();
                block.offset(x, y);

                let mat = mat.mul_mat3(&Mat3::from_translation(Vec2::new(x as f32, y as f32)));

                let color_ratio = ((position.zoom_level - 0.8) / 0.2).clamp(0.0, 1.0);
                let invisible = Color::new(0.0, 0.0, 0.0, 0.0);
                let color0 = lerp_colors(BLUE, invisible, color_ratio);
                let color1 = lerp_colors(GREEN, BLUE, color_ratio);
                let color2 = lerp_colors(invisible, GREEN, color_ratio);
                let mut subcolors = vec![color1];
                if color2.a > 0.0 {
                    subcolors.push(color2);
                }

                draw_block(&mat, &block, color0, &subcolors);
            }
        }

        draw_text(
            &format!("Arrows,Z,X {:?}", position),
            20.0,
            20.0,
            15.0,
            BLUE,
        );

        next_frame().await
    }
}

#[cfg(test)]
mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_view_position_zoom() {
        let mut pos = ViewPosition {
            block: BlockAddress(vec![IVec2::new(1, 1)]),
            position: Vec2::new(0.5, 0.5),
            zoom_level: 0.1,
        };
        pos.zoom(1.1);
        assert_eq!(
            pos,
            ViewPosition {
                block: BlockAddress(vec![IVec2::new(1, 1), IVec2::new(5, 5)]),
                position: Vec2::new(0.0, 0.0),
                zoom_level: 0.2,
            }
        );
        pos.zoom(-1.1);
        assert_eq!(
            pos,
            ViewPosition {
                block: BlockAddress(vec![IVec2::new(1, 1)]),
                position: Vec2::new(0.5, 0.5),
                zoom_level: 0.1,
            }
        );
        pos.zoom(-100.0);
        assert_eq!(
            pos,
            ViewPosition {
                block: BlockAddress(vec![]),
                position: Vec2::new(0.15, 0.15),
                zoom_level: 0.0,
            }
        );
    }

    #[test]
    fn test_view_position_offset() {
        let mut pos = ViewPosition {
            block: BlockAddress(vec![IVec2::new(1, 1)]),
            position: Vec2::new(0.5, 0.5),
            zoom_level: 0.0,
        };

        pos.offset(1.1, 0.0);
        assert_eq!(
            pos,
            ViewPosition {
                block: BlockAddress(vec![IVec2::new(2, 1)]),
                position: Vec2::new(0.6, 0.5),
                zoom_level: 0.0
            }
        );
        pos.offset(0.0, 1.1);
        assert_eq!(
            pos,
            ViewPosition {
                block: BlockAddress(vec![IVec2::new(2, 2)]),
                position: Vec2::new(0.6, 0.6),
                zoom_level: 0.0
            }
        );

        pos.offset(-2.1, -2.1);
        assert_eq!(
            pos,
            ViewPosition {
                block: BlockAddress(vec![IVec2::new(0, 0)]),
                position: Vec2::new(0.5, 0.5),
                zoom_level: 0.0
            }
        );
    }
}
