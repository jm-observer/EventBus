use proc_macro::TokenStream;
use quote::quote;
use syn::__private::TokenStream2;
use syn::{Item, Type};

#[proc_macro_derive(Merge)]
pub fn merge_derive(input: TokenStream) -> TokenStream {
    general_merge(input.into()).unwrap().into()
}

fn general_merge(code: TokenStream2) -> Result<TokenStream2, String> {
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
                fn merge(event: for_event_bus::BusEvent) -> Result<Self, BusError>
                where
                    Self: Sized,
                {
                    use for_event_bus::upcast;
                    let event = upcast(event);
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

#[proc_macro_derive(Worker)]
pub fn worker_derive(input: TokenStream) -> TokenStream {
    general_worker(input.into()).unwrap().into()
}

fn general_worker(code: TokenStream2) -> Result<TokenStream2, String> {
    let ident = if let Ok(Item::Struct(item_enum)) = syn::parse2(code.clone()) {
        item_enum.ident
    } else if let Ok(Item::Enum(item_enum)) = syn::parse2(code) {
        item_enum.ident
    } else {
        return Err("only support enum/struct to merge event!".to_string());
    };
    let name = ident.to_string();
    let end = quote!(
        impl ToWorker for #ident {
            fn name() -> String {
                #name.to_string()
            }
        }
    );
    Ok(end)
}

#[proc_macro_derive(Event)]
pub fn event_derive(input: TokenStream) -> TokenStream {
    general_event(input.into()).unwrap().into()
}

fn general_event(code: TokenStream2) -> Result<TokenStream2, String> {
    let ident = if let Ok(Item::Struct(item_enum)) = syn::parse2(code.clone()) {
        item_enum.ident
    } else if let Ok(Item::Enum(item_enum)) = syn::parse2(code) {
        item_enum.ident
    } else {
        return Err("only support enum/struct to merge event!".to_string());
    };
    let name = ident.to_string();
    let end = quote!(
        impl for_event_bus::Event for #ident {
            fn name() -> String {
                #name.to_string()
            }
        }
    );
    Ok(end)
}
