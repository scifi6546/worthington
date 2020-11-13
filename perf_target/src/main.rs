use table::{DatabaseTable, Insertable};
fn main() {
    let mut t: DatabaseTable<u32> = DatabaseTable::new();
    let mut v = vec![];
    for i in 0..1_000 {
        v.push((t.insert(i), i));
    }
    for (key, value) in v.iter() {
        assert_eq!(t.get(key.clone()).ok().unwrap(), value.clone());
    }
}
