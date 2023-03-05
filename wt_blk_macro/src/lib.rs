extern crate proc_macro;
use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};
use quote::quote;


#[proc_macro]
pub fn time(input: TokenStream) -> TokenStream {
   let input: String = input.to_string().parse().unwrap();
   let tokens = quote! {
         let start = std::time::Instant::now();
         #input;
         println!("{}:{} {:?}", file!{}, line!{},start.elapsed());
         drop(start);
    };

   tokens.into()
}
