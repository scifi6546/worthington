use table::{DatabaseTable, InsertableDyn};
fn main() {
    let mut t: DatabaseTable = DatabaseTable::new();
    let mut v = vec![];
    for i in 0..1_000_000 {
        v.push((t.insert(i as u32), i));
    }
    for (key, value) in v.iter() {
        assert_eq!(
            t.get(key.clone(), |b| u32::from_le_bytes([
                b[0], b[1], b[2], b[3]
            ]))
            .ok()
            .unwrap(),
            value.clone()
        );
    }
}
