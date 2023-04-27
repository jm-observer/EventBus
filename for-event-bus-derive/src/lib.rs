use proc_macro::TokenStream;
use quote::quote;
use syn::__private::TokenStream2;
use syn::{Item, Type};

#[proc_macro_derive(Merge)]
pub fn merge_derive(input: TokenStream) -> TokenStream {
    general(input.into()).unwrap().into()
}

fn general(code: TokenStream2) -> Result<TokenStream2, String> {
    if let Ok(Item::Enum(item_enum)) = syn::parse2(code) {
        let ident = item_enum.ident;
        let mut tokens = Vec::new();
        let mut type_ids = Vec::new();
        for var in item_enum.variants {
            let var_ident = var.ident;
            for field in var.fields {
                if let Type::Path(path) = field.ty {
                    let segments = path.path.segments;
                    tokens.push(quote!(
                    if let Ok(a_event) = event.clone().downcast::<#segments>() {
                        Ok(Self::#var_ident(a_event.as_ref().clone()))
                    }));

                    type_ids.push(quote!(
                    TypeId::of::<#segments>()
                    ));
                } else {
                    return Err(
                        "field of enum only support type of path, e.g: Close(supper::Close)"
                            .to_string(),
                    );
                }
            }
        }

        let end = quote!(
            impl Merge for #ident {
                fn merge(event: for_event_bus::Event) -> Result<Self, BusError>
                where
                    Self: Sized,
                {
                    #(#tokens)else* else {
                        Err(BusError::DowncastErr)
                    }
                }

                fn subscribe_types() -> Vec<TypeId> {
                    vec![#(#type_ids),*]
                }
            }
        );
        Ok(end)
    } else {
        Err("only support enum to merge event!".to_string())
    }
}
