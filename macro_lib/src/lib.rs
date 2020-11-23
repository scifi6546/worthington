use graph::Node;

extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::quote;
use syn;
use syn::{parse_macro_input, DeriveInput};
#[proc_macro_derive(NodeT)]
pub fn node(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree

    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let tree = quote! {impl Node for #name{


        const HASH: NodeHash=NodeHash{hash:0};
    fn get_data(
        &self,
    ) -> (
        Vec<(NodeElementHash, Box<dyn InsertableDyn>)>,
        Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
    ){todo!()}
    fn from_data(
        sized: Vec<(NodeElementHash, Vec<u8>)>,
        variable: Vec<(NodeElementHash, Vec<u8>)>,
    ) -> Self{todo!()}}}
    .into();

    //let mut tree = TokenStream::new();
    //tree.extend(vec![
    //    TokenTree::Ident(Ident::new("impl", Span::call_site())),
    //    TokenTree::Ident(Ident::new("Node", Span::call_site())),
    //    TokenTree::Ident(Ident::new("for", Span::call_site())),
    //    TokenTree::Ident(Ident::new("Bar", Span::call_site())),
    //    TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::new())),
    //]);
    return tree;
}
#[cfg(test)]
mod tests {
    #[node]
    struct Bar {}

    #[test]
    fn empty() {
        let b = Bar {};
        bar.to_data();
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
