use sha2::{Digest, Sha256};
extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident as procIdent, Span, TokenStream, TokenTree};
use quote::quote;
use syn::Data;
use syn::{parse_macro_input, DataStruct, DeriveInput, Fields, Path, Type};
fn parse_struct(s: DataStruct) -> Vec<syn::Ident> {
    match s.fields {
        Fields::Named(named) => {
            let mut v: Vec<syn::Ident> = vec![];
            named
                .named
                .iter()
                .map(|f| f.ident.clone().unwrap())
                .collect()
        }
        Fields::Unnamed(_) => todo!(),
        Fields::Unit => todo!(),
    }
}
fn get_paths(s: DataStruct) -> Vec<Path> {
    s.fields
        .into_iter()
        .map(|field| match field.ty {
            Type::Array(_) => panic!("invalid type: array"),
            Type::BareFn(_) => panic!("invalid type: barefn"),
            Type::Path(p) => p.path,

            Type::Verbatim(s) => panic!("invalid type: Verbatim"),
            _ => panic!("invalid type"),
        })
        .collect()
}
#[proc_macro_derive(GraphInsertable)]
pub fn node(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree

    let mut hasher = Sha256::new();
    let ast = parse_macro_input!(input as DeriveInput);
    let (sized, types) = match ast.data {
        Data::Struct(s) => (parse_struct(s.clone()), get_paths(s)),
        Data::Enum(_) => todo!(),
        Data::Union(_) => todo!(),
    };
    hasher.update(ast.ident.to_string());
    for field in sized.iter() {
        hasher.update(field.to_string());
    }
    let hash_bytes = hasher.finalize();
    let hash = usize::from_le_bytes([
        hash_bytes[0],
        hash_bytes[1],
        hash_bytes[2],
        hash_bytes[3],
        hash_bytes[4],
        hash_bytes[5],
        hash_bytes[6],
        hash_bytes[7],
    ]);

    let name = &ast.ident;
    let tree = quote! {impl Node for #name{
        const SELF_HASH: NodeHash=NodeHash{hash:#hash};
        fn get_sized_hashes() -> Vec<NodeElementHash> {
            let mut out = vec![];
            #({
                out.append(&mut #types::get_sized_hashes());

            })*
            return out;
        }
        fn get_variable_hashes() -> Vec<NodeElementHash> {
            let mut out = vec![];
            #({
                out.append(&mut #types::get_variable_hashes());

            })*
            return out;
        }
        fn get_data(&self)->(
            Vec<(NodeElementHash, Box<dyn InsertableDyn>)>,
            Vec<(NodeElementHash, Box<dyn VariableSizeInsert>)>,
            ){
            let mut sized_out = vec![];
            let mut variable_out = vec![];
            #(
                {
                    let (mut sized,mut var) =self.
                #sized
                        .get_data();
                    let mut sized_new = sized.iter().map(|(hash,i)|

                        (NodeElementHash{hash:hash.hash&#hash},i.clone())).collect();
                    let mut var_new = var.iter().map(|(hash,i)|(NodeElementHash{hash:hash.hash&#hash},i.clone())).collect();
                    sized_out.append(&mut sized_new);
                    variable_out.append(&mut var_new);
                }
            )*
                return (sized_out,variable_out)

            }
        fn from_data(
            sized: Vec<(NodeElementHash, Vec<u8>)>,
            variable: Vec<(NodeElementHash, Vec<u8>)>,
        ) -> Self{
            #(
                let #sized = {
                    let mut sized_data = vec![];
                    for (hash,data) in sized.iter(){
                        for client_hash in Self::get_sized_hashes().iter(){
                            if hash==client_hash{
                                sized_data.push((hash.clone(),data.clone()));

                            }

                        }

                    }
                    let mut variable_data = vec![];
                    for (hash,data) in variable.iter(){
                        for client_hash in Self::get_variable_hashes().iter(){
                            if hash==client_hash{
                                variable_data.push((hash.clone(),data.clone()));

                            }

                        }

                    }
                    #types::from_data(sized_data,variable_data)
                };

            )*
                Self{
                    #(
                        #sized

                    ),*

                }


            }}

    }
    .into();

    return tree;
}
// Hashes identifier, panics if input is not identifier
#[proc_macro]
pub fn hash(input: TokenStream) -> TokenStream {
    let token = input.into_iter().next().unwrap();
    let hash_bytes = match token {
        TokenTree::Ident(i) => {
            let mut hash = Sha256::new();
            hash.update(i.to_string());
            hash.finalize()
        }
        _ => panic!(),
    };
    let hash = usize::from_le_bytes([
        hash_bytes[0],
        hash_bytes[1],
        hash_bytes[2],
        hash_bytes[3],
        hash_bytes[4],
        hash_bytes[5],
        hash_bytes[6],
        hash_bytes[7],
    ]);
    let t = quote! {#hash};
    t.into()
}
