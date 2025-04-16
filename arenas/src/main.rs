use arenas::{SegmentTreeWithRealId, Id};
fn main() {
    dbg!(size_of::<Option<Id>>());
    let mut tree = SegmentTreeWithRealId::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(tree.range_sum(0, 1), 3);
    assert_eq!(tree.range_sum(0, 2), 6);
    assert_eq!(tree.range_sum(7, 8), 17);
    tree.setitem(3, &7);
    assert_eq!(tree.range_sum(2, 5), 21);
}
