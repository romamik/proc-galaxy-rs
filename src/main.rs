extern crate float_cmp;
extern crate more_asserts;
extern crate rand;

use float_cmp::*;
use macroquad::prelude::*;
use more_asserts::*;
//use rand::{rngs::StdRng, RngCore, SeedableRng};

const SUBBLOCK_COUNT: i32 = 10;
const SUBBLOCK_COUNT_F: f32 = SUBBLOCK_COUNT as f32;

/*
Each block is subdivided into SUBBLOCK_COUNT * SUBBLOCK_COUNT subblocks
Each block can be addressed as vector of IVec2:
    root block is []
    some subblock of root block is [(1,1)]
    subblock of that subblock is [(1,1), (2,2)]
    and so on
*/
#[derive(Debug, Clone, PartialEq)]
struct BlockAddress(Vec<IVec2>);

impl BlockAddress {
    pub fn offset(&mut self, offset_x: i32, offset_y: i32) {
        let addr = &mut self.0;
        let mut i = addr.len();
        let mut dx = offset_x;
        let mut dy = offset_y;

        while i > 0 && (dx != 0 || dy != 0) {
            i -= 1;
            let x = addr[i].x + dx;
            let y = addr[i].y + dy;
            addr[i].x = ((x % SUBBLOCK_COUNT) + SUBBLOCK_COUNT) % SUBBLOCK_COUNT;
            addr[i].y = ((y % SUBBLOCK_COUNT) + SUBBLOCK_COUNT) % SUBBLOCK_COUNT;
            dx = (x - addr[i].x) / SUBBLOCK_COUNT;
            dy = (y - addr[i].y) / SUBBLOCK_COUNT;
        }
    }

    pub fn get_zoom(&self) -> i32 {
        self.0.len() as i32
    }

    pub fn get_last_block_pos(&self) -> (i32, i32) {
        let last = self.0.last().unwrap();
        (last.x, last.y)
    }

    pub fn zoom_in(&mut self, block_x: i32, block_y: i32) {
        assert_ge!(block_x, 0);
        assert_le!(block_x, SUBBLOCK_COUNT);
        assert_ge!(block_y, 0);
        assert_le!(block_y, SUBBLOCK_COUNT);

        self.0.push(IVec2::new(block_x, block_y));
    }

    pub fn zoom_out(&mut self) {
        assert!(!self.0.is_empty());
        self.0.pop();
    }
}

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

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut position = ViewPosition {
        block: BlockAddress(vec![]),
        position: Vec2::new(0.5, 0.5),
        zoom_level: 0.0,
    };

    loop {
        clear_background(RED);

        let mut dpos = Vec2::new(0.0, 0.0);

        if let Some(key) = get_last_key_pressed() {
            match key {
                KeyCode::Left => dpos.x -= 1.0,
                KeyCode::Right => dpos.x += 1.0,
                KeyCode::Down => dpos.y += 1.0,
                KeyCode::Up => dpos.y -= 1.0,
                KeyCode::Z => position.zoom(0.1),
                KeyCode::X => position.zoom(-0.1),
                KeyCode::Escape => break,
                _ => {}
            }
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

        position.position += mat.inverse().transform_vector2(dpos * sw * 0.01);

        // and subbblocks
        for i in 0..SUBBLOCK_COUNT {
            for j in 0..SUBBLOCK_COUNT {
                let mat = mat
                    .mul_mat3(&Mat3::from_scale(Vec2::new(
                        1.0 / SUBBLOCK_COUNT_F,
                        1.0 / SUBBLOCK_COUNT_F,
                    )))
                    .mul_mat3(&Mat3::from_translation(Vec2::new(i as f32, j as f32)));
                let p00 = mat.transform_point2(Vec2::new(0.0, 0.0));
                let p10 = mat.transform_point2(Vec2::new(1.0, 0.0));
                let p01 = mat.transform_point2(Vec2::new(0.0, 1.0));
                let p11 = mat.transform_point2(Vec2::new(1.0, 1.0));
                draw_line(p00.x, p00.y, p10.x, p10.y, 1.0, GREEN);
                draw_line(p10.x, p10.y, p11.x, p11.y, 1.0, GREEN);
                draw_line(p11.x, p11.y, p01.x, p01.y, 1.0, GREEN);
                draw_line(p01.x, p01.y, p00.x, p00.y, 1.0, GREEN);
            }
        }

        // draw block
        let p00 = mat.transform_point2(Vec2::new(0.0, 0.0));
        let p10 = mat.transform_point2(Vec2::new(1.0, 0.0));
        let p01 = mat.transform_point2(Vec2::new(0.0, 1.0));
        let p11 = mat.transform_point2(Vec2::new(1.0, 1.0));
        draw_line(p00.x, p00.y, p10.x, p10.y, 1.0, BLUE);
        draw_line(p10.x, p10.y, p11.x, p11.y, 1.0, BLUE);
        draw_line(p11.x, p11.y, p01.x, p01.y, 1.0, BLUE);
        draw_line(p01.x, p01.y, p00.x, p00.y, 1.0, BLUE);

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

    #[test]
    fn test_block_address_zoom() {
        let mut addr = BlockAddress(vec![]);
        addr.zoom_in(1, 1);
        assert_eq!(addr, BlockAddress(vec![IVec2::new(1, 1)]));
        addr.zoom_out();
        assert_eq!(addr, BlockAddress(vec![]));
    }

    #[test]
    fn test_block_address_offset() {
        let mut addr = BlockAddress(vec![IVec2::new(5, 5), IVec2::new(0, 0)]);
        addr.offset(1, 1);
        assert_eq!(addr, BlockAddress(vec![IVec2::new(5, 5), IVec2::new(1, 1)]));
        addr.offset(-2, -2);
        assert_eq!(
            addr,
            BlockAddress(vec![
                IVec2::new(4, 4),
                IVec2::new(SUBBLOCK_COUNT - 1, SUBBLOCK_COUNT - 1)
            ])
        );

        let mut addr = BlockAddress(vec![IVec2::new(5, 5), IVec2::new(0, 0)]);
        addr.offset(SUBBLOCK_COUNT + 1, 0);
        assert_eq!(addr, BlockAddress(vec![IVec2::new(6, 5), IVec2::new(1, 0)]));
        addr.offset(0, SUBBLOCK_COUNT + 1);
        assert_eq!(addr, BlockAddress(vec![IVec2::new(6, 6), IVec2::new(1, 1)]));

        let mut addr = BlockAddress(vec![IVec2::new(5, 5), IVec2::new(0, 0)]);
        addr.offset(-SUBBLOCK_COUNT * 2 - 1, 0);
        assert_eq!(
            addr,
            BlockAddress(vec![IVec2::new(2, 5), IVec2::new(SUBBLOCK_COUNT - 1, 0)])
        );
        addr.offset(0, -SUBBLOCK_COUNT * 2 - 1);
        assert_eq!(
            addr,
            BlockAddress(vec![
                IVec2::new(2, 2),
                IVec2::new(SUBBLOCK_COUNT - 1, SUBBLOCK_COUNT - 1)
            ])
        );
    }
}
