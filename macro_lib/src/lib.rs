use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
#[proc_macro]
pub fn database_schema(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree

    for token in input {
        match token {
            TokenTree::Group(_) => (println!("in group")),
            TokenTree::Ident(_) => (),
            TokenTree::Punct(_) => (),
            TokenTree::Literal(_) => (),
        }
    }

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        // ...
    };

    let mut tree = TokenStream::new();
    return tree;
}
#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        //#[macro_use]
        //extern crate graph;
        //database_schema!(Person { name: String }, Pet { species: String });
        //let db = Database::new();
        //let person_key: Box<Person::key> = db.Person.insert(Person::new {
        //    name: "Bill".to_string(),
        //});
        //let jill_key: Box<Person::key> = db.Person.insert(Person::new {
        //    name: "Jill".to_string(),
        //});
        //let pet_key: Box<Pet::Key> = db.Pet.insert(Pet::new {
        //    species: "dog".to_string(),
        //});
        //db.link(person_key, pet_key);
        //for pet in db.Person.Pet {
        //    assert_eq!(pet.species, "dog".to_string());
        //}
    }
}
