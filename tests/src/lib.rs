use graph::prelude::*;
#[macro_use]
extern crate macro_lib;
#[derive(GraphInsertable, Debug, PartialEq)]
struct Empty {}
#[derive(GraphInsertable, Debug, PartialEq)]
struct S {
    name: String,
}
#[derive(GraphInsertable, Debug, PartialEq, Clone)]
struct Person {
    age: u64,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::Database;
    #[test]
    fn it_works() {
        let mut db = Database::new();
        let e = db.insert(Empty {});
        assert_eq!(db.get::<Empty>(e).unwrap(), Empty {});
    }
    #[test]
    fn insert_string() {
        let mut db = Database::new();
        let e = db.insert(S {
            name: "bar".to_string(),
        });
        assert_eq!(
            db.get::<S>(e).unwrap(),
            S {
                name: "bar".to_string()
            }
        );
    }
    #[test]
    fn insert_person() {
        let mut db = Database::new();
        let p_obj = Person {
            name: "Bill".to_string(),
            age: 5,
        };
        let p_key = db.insert(p_obj.clone());
        assert_eq!(db.get::<Person>(p_key).unwrap(), p_obj);
    }
}
