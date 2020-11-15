mod Items {
    pub struct Foo {}
    pub struct Bar {}
}
pub enum ItemEntry {
    Foo(Items::Foo),
    Bar(Items::Bar),
}
#[derive(Clone)]
pub struct Key {}

#[macro_export]
macro_rules! database {
    ($($type:ty,$ident:ident),*) => {
        enum Item {
            $(
            $ident($type),
            )*
        }

    };
}
pub trait Node {
    //hash of the database name
    const Hash: usize;
}
pub struct Database {}
impl Database {
    pub fn new() -> Self {
        unimplemented!()
    }
    pub fn insert<Data: Node>(&mut self, data: Data) -> Key {
        unimplemented!()
    }
    pub fn connect(&mut self, key1: Key, key2: Key) {
        unimplemented!()
    }
    pub fn get_connected(&self, key: Key) -> Vec<Key> {
        unimplemented!()
    }
    pub fn get<Data: Node>(&self, key: Key) -> Option<Data> {
        unimplemented!()
    }
}

/// ```
/// #[macro_use] extern crate graph;
/// struct Person{name:String}
/// struct Pet{species:String}
/// database!(Person,Person,Pet,Pet);
/// let mut db = db::new();
/// let k = db.insert(Item::Person(Person{name:"Bill".to_string()}));
/// let pk = db.insert(Item::Pet(Pet{species:"dog".to_string()}));
/// ```
/// Or?
/// ```
/// #[macro_use] extern crate graph;
/// database_schema!(Person{name:String},Pet{species:String});
/// let db = Database::new();
/// let person_key:Box<Person::key> = db.Person.insert(Person::new{name:"Bill".to_string()})
/// let jill_key:Box<Person::key> = db.Person.insert(Person::new{name:"Jill".to_string()})
/// let pet_key:Box<Pet::Key> = db.Pet.insert(Pet::new{species:"dog".to_string()});
/// db.link(person_key,pet_key);
/// for pet in db.Person.Pet{
///     assert_eq!(pet.species,"dog".to_string());
///
/// }
///
///
/// ```
mod foo {
    fn t() {
        let _t = {
            mod bar {
                pub fn foos() {
                    println!("bar?")
                }
            }
            bar::foos()
        };
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(PartialEq, Debug)]
    struct Person {
        name: String,
    }
    #[derive(PartialEq, Debug)]
    struct Pet {
        species: String,
    }
    impl Node for Person {
        const Hash: usize = 0;
    }
    impl Node for Pet {
        const Hash: usize = 1;
    }
    #[test]
    fn basic_api() {
        let mut db = Database::new();
        let bill = db.insert(Person {
            name: "Bill".to_string(),
        });
        let dog = db.insert(Pet {
            species: "dog".to_string(),
        });
        db.connect(bill, dog.clone());
        assert_eq!(
            db.get::<Person>((db.get_connected(dog)[0]).clone())
                .unwrap(),
            Person {
                name: "Bill".to_string()
            }
        );

        assert_eq!(2 + 2, 4);
    }
}
