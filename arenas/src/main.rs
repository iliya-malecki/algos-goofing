use arenas::{Id, SegmentTreeWithEphemeralId};

fn main() {
    dbg!(size_of::<Option<Id>>());
    let mut tree = SegmentTreeWithEphemeralId::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(tree.range_sum(0, 1), 3);
    assert_eq!(tree.range_sum(0, 2), 6);
    assert_eq!(tree.range_sum(7, 8), 17);
    tree.setitem(3, &7);
    assert_eq!(tree.range_sum(2, 5), 21);
    let mut tree = SegmentTreeWithEphemeralId::new(vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32, 33,
    ]);
    tree.setitem(5, &71);
    tree.setitem(9, &3);
    assert_eq!(tree.range_sum(3, 19), 262);
}
