extern crate more_asserts;

use macroquad::prelude::*;
use more_asserts::*;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub const SUBBLOCK_COUNT: i32 = 10;
pub const SUBBLOCK_COUNT_F: f32 = SUBBLOCK_COUNT as f32;

/*
Each block is subdivided into SUBBLOCK_COUNT * SUBBLOCK_COUNT subblocks
Each block can be addressed as vector of IVec2:
    root block is []
    some subblock of root block is [(1,1)]
    subblock of that subblock is [(1,1), (2,2)]
    and so on
*/
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct BlockAddress(pub Vec<IVec2>);

impl BlockAddress {
    pub fn get_name(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        base64::encode(hasher.finish().to_ne_bytes())
    }

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
        if let Some(last) = self.0.last() {
            (last.x, last.y)
        } else {
            (0, 0)
        }
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

#[cfg(test)]
mod tests {

    use super::*;

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
