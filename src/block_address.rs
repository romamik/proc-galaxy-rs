extern crate more_asserts;

use macroquad::prelude::*;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/*
Space is organized in blocks, each block having sub blocks. Coordinates of sub blocks are in range [-HALF_SIZE..HALF_SIZE].
Block address allows to find a block in space, starting from the root block.
Root block is {parent: 0, child: []}
Root block's coordinates in it's parent are (0, 0)
The procedure is as follows:
    1. First we get parent of current block `address.parent` times.
    2. Than we get child of the current block using coordinates from every element of `address.child`.
Addresses must be unique, so that it is not allowed visit any block twice during this navigation.
*/

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct BlockAddress {
    parent: u32,
    child: Vec<IVec2>,
}

impl BlockAddress {
    pub const HALF_SIZE: i32 = 5;
    pub const SIZE: i32 = Self::HALF_SIZE * 2 + 1;
    pub const ROOT: BlockAddress = BlockAddress {
        parent: 0,
        child: vec![],
    };

    pub fn get_name(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        base64::encode(hasher.finish().to_ne_bytes())
    }

    // changes self to parent of self, returns coordinates of previous self in parent
    pub fn to_parent(&mut self) -> IVec2 {
        match self.child.pop() {
            Some(last) => last,
            None => {
                self.parent += 1;
                IVec2::ZERO
            }
        }
    }

    // changes self to child of self with given coordinates
    // if coordinates are out of range set self to child of the corresponding sibling
    pub fn to_child(&mut self, mut coord: IVec2) {
        let parent_offset = IVec2::new(
            Self::put_in_range(&mut coord.x),
            Self::put_in_range(&mut coord.y),
        );
        self.to_sibling(parent_offset);
        if self.parent > 0 && self.child.len() == 0 && coord.eq(&IVec2::ZERO) {
            self.parent -= 1;
        } else {
            self.child.push(coord);
        }
    }

    // changes self to address of block on same level
    pub fn to_sibling(&mut self, offset: IVec2) {
        if offset != IVec2::ZERO {
            let mut coord = self.to_parent();
            coord += offset;
            self.to_child(coord);
        }
    }

    // given val in block space, puts val in range [-HALF_SIZE..HALF_SIZE] and returns offset in parent space
    fn put_in_range(val: &mut i32) -> i32 {
        const HALF_SIZE: i32 = BlockAddress::HALF_SIZE;
        const SIZE: i32 = BlockAddress::SIZE;

        let new_val = ((*val + HALF_SIZE) % SIZE + SIZE) % SIZE - HALF_SIZE;
        let remainder = (*val - new_val) / SIZE;
        *val = new_val;
        remainder
    }
}

#[cfg(test)]
mod tests {

    use itertools::Itertools;

    use super::*;
    const HALF_SIZE: i32 = BlockAddress::HALF_SIZE;
    const SIZE: i32 = BlockAddress::SIZE;

    fn make_ba(parent: u32, child: &[(i32, i32)]) -> BlockAddress {
        BlockAddress {
            parent: parent,
            child: child.iter().map(|&(x, y)| IVec2::new(x, y)).collect(),
        }
    }

    #[test]
    fn test_put_in_range() {
        for (start_val, expect_val, expect_parent_offset) in [
            (0, 0, 0),
            (HALF_SIZE, HALF_SIZE, 0),
            (HALF_SIZE + 1, -HALF_SIZE, 1),
            (SIZE, 0, 1),
            (SIZE * SIZE + 1, 1, SIZE),
            (-HALF_SIZE, -HALF_SIZE, 0),
            (-HALF_SIZE - 1, HALF_SIZE, -1),
            (-SIZE, 0, -1),
            (-SIZE * SIZE - 1, -1, -SIZE),
        ] {
            let mut val = start_val;
            let parent_offset = BlockAddress::put_in_range(&mut val);
            assert_eq!(val, expect_val, "start_val: {}", start_val);
            assert_eq!(
                parent_offset, expect_parent_offset,
                "start_val: {}",
                start_val
            );
        }
    }

    #[test]
    fn test_to_parent() {
        for (start_block, expect_block, (expect_x, expect_y)) in [
            (BlockAddress::ROOT, make_ba(1, &[]), (0, 0)),
            (make_ba(1, &[]), make_ba(2, &[]), (0, 0)),
            (make_ba(0, &[(0, 0)]), BlockAddress::ROOT, (0, 0)),
            (
                make_ba(0, &[(HALF_SIZE, HALF_SIZE)]),
                BlockAddress::ROOT,
                (HALF_SIZE, HALF_SIZE),
            ),
            (
                make_ba(2, &[(HALF_SIZE, HALF_SIZE)]),
                make_ba(2, &[]),
                (HALF_SIZE, HALF_SIZE),
            ),
        ] {
            let mut block = start_block.clone();
            let coord = block.to_parent();
            assert_eq!(block, expect_block, "start_block: {:?}", start_block);
            assert_eq!(
                coord,
                IVec2::new(expect_x, expect_y),
                "start_block: {:?}",
                start_block
            );
        }
    }

    #[test]
    fn test_to_child() {
        fn test(
            start_parent: u32,
            start_child: &[(i32, i32)],
            x: i32,
            y: i32,
            expect_parent: u32,
            expect_child: &[(i32, i32)],
        ) {
            let start = make_ba(start_parent, start_child);
            let expect = make_ba(expect_parent, expect_child);
            let coord = IVec2::new(x, y);
            let mut block = start.clone();
            block.to_child(IVec2::new(x, y));
            assert_eq!(
                block, expect,
                "start_block: {:?}, coord: {}, {}",
                start, x, y
            );
        }

        test(0, &[], 0, 0, 0, &[(0, 0)]);
        test(0, &[], HALF_SIZE, 0, 0, &[(HALF_SIZE, 0)]);
        test(0, &[], 0, HALF_SIZE, 0, &[(0, HALF_SIZE)]);
        test(0, &[], HALF_SIZE + 1, 0, 1, &[(1, 0), (-HALF_SIZE, 0)]);
        test(0, &[], 0, HALF_SIZE + 1, 1, &[(0, 1), (0, -HALF_SIZE)]);
        test(0, &[], SIZE, 0, 1, &[(1, 0), (0, 0)]);
        test(0, &[], 0, SIZE, 1, &[(0, 1), (0, 0)]);
        test(0, &[], SIZE * SIZE, 0, 2, &[(1, 0), (0, 0), (0, 0)]);
        test(0, &[], 0, SIZE * SIZE, 2, &[(0, 1), (0, 0), (0, 0)]);
    }

    #[test]
    fn test_to_sibling() {
        fn test(start: &BlockAddress, offset: &(i32, i32), expected: &BlockAddress) {
            let mut block = start.clone();
            block.to_sibling(IVec2::new(offset.0, offset.1));
            assert_eq!(
                block, *expected,
                "start_block: {:?}, offset: {:?}",
                start, offset
            );
        }
        fn test2(
            start_parent: u32,
            start_child: &[i32],
            offset: i32,
            expected_parent: u32,
            expected_child: &[i32],
        ) {
            for (kx, ky) in (-1..=1)
                .permutations(2)
                .map(|v| v.into_iter().collect_tuple().unwrap())
            {
                let make_child =
                    |slice: &[i32]| slice.iter().map(|&v| (kx * v, ky * v)).collect::<Vec<_>>();

                let start = make_ba(start_parent, &make_child(start_child));
                let offset_vec = (kx * offset, ky * offset);
                let neg_offset_vec = (-kx * offset, -ky * offset);
                let expected = make_ba(expected_parent, &make_child(expected_child));
                test(&start, &offset_vec, &expected);
                test(&expected, &neg_offset_vec, &start);
            }
        }
        test2(0, &[], 0, 0, &[]);
        test2(0, &[], 1, 1, &[1]);
        test2(0, &[], HALF_SIZE, 1, &[HALF_SIZE]);
        test2(0, &[], HALF_SIZE + 1, 2, &[1, -HALF_SIZE]);
        test2(0, &[], SIZE, 2, &[1, 0]);
        test2(0, &[], SIZE, 2, &[1, 0]);
    }

    #[test]
    fn test_name() {
        let mut block_0 = BlockAddress::ROOT.clone();
        let mut block_1 = BlockAddress::ROOT.clone();
        assert_eq!(block_0.get_name(), block_1.get_name());

        block_0.to_sibling(IVec2::new(1, 0));
        assert_ne!(block_0.get_name(), block_1.get_name());

        block_0.to_sibling(IVec2::new(-1, 0));
        assert_eq!(block_0.get_name(), block_1.get_name());

        block_0.to_sibling(IVec2::new(-1, 0));
        block_0.to_sibling(IVec2::new(-1, 0));
        block_1.to_sibling(IVec2::new(-2, 0));
        assert_eq!(block_0.get_name(), block_1.get_name());
    }
}
