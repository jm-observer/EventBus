use custom_utils::logger::debug;
use custom_utils::parse_file;
use quote::quote;
use syn::{Fields, File, Item, Type};

#[tokio::main]
async fn main() {
    custom_utils::logger::logger_stdout_debug();
    let codes = r#"
    fn c() {
    if a == b {
    } else if  a== c {
    } else {
    }
    }
"#;
    let token = parse_file(codes).await.unwrap();
    debug!("{:?}", token);
    for item in token.items {
        if let Item::Fn(item_enum) = item {
            let ident = item_enum.block;
            debug!("{:?}", ident);
            // for var in item_enum.variants {
            //     let var_ident = var.ident;
            //     debug!("{}", var_ident);
            //     for field in var.fields {
            //     }
            // }
        }
    }
}
