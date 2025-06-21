extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn debug_trace(args: TokenStream, input: TokenStream) -> TokenStream {
    let prefix = if args.is_empty() {
        "DEBUG".to_string()
    } else {
        args.to_string().trim_matches('"').to_string()
    };

    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;


    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            let __debug_result = (|| #fn_block)();

            #[cfg(feature = "debug-output")]
            {
                println!("[{} @ line: {}]", #prefix, line!());
                let lhs = format!("  └── {}() -> ", stringify!(#fn_name));
                let debug_str = format!("{:?}", __debug_result);
                let lines: Vec<&str> = debug_str.lines().collect();

                if lines.len() > 1 {
                    println!("{}{}", lhs, lines[0]);
                }
                // Add padding equal to lhs length to all subsequent lines
                let padding = " ".repeat(lhs.len());
                for line in lines.iter().skip(1) {
                    println!("{}{}", padding, line);
                }
            }
            __debug_result
        }
    };

    TokenStream::from(expanded)
}
