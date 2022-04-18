//! This module exposes some custom "ergonomic" derive macros for aiding in
//! dispatch to subcommands.
//!
//! This is very much a work in progress
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta, MetaList, NestedMeta, Variant};

fn external_branch(variant: &Variant) -> Option<proc_macro2::TokenStream> {
    let name = &variant.ident;

    variant
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("clap") && !a.tokens.is_empty())
        .find(|a| {
            // so... this is a bit crazy, but the resources for learning proc
            // macros are a little sparse, and I'm not very familiar with them
            if let Ok(Meta::List(MetaList { nested, .. })) = a.parse_meta() {
                nested.iter().any(|nest| {
                    if let NestedMeta::Meta(Meta::Path(p)) = nest {
                        p.is_ident("external_subcommand")
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        })
        .map(|_| {
            quote! {
                Self::#name(args) => {
                    use anyhow::bail;
                    use clap::crate_name;
                    use durandal_core::external::ExternalCommand;

                    if args.is_empty() {
                        bail!("Unexpected empty external subcommand vector")
                    }

                    ExternalCommand::new()
                        .prefix(crate_name!())
                        .name(&args[0])
                        .args(&args[1..])
                        .build()?
                        .run()
                    }
            }
        })
}

/// This macro reduces the boilerplate in dispatching cli subcommand executions.
///
/// Because this macro generates the dispatch logic for enum variants, it
/// naturally also only is valid for enums.
///
/// This macro _only_ works with the `CliCommand` trait exposed by
/// `durandal-core`. If you need to dispatch to subcommands implementing
/// `CliMetaCommand`, then use the [CliMetaDispatch] macro instead.
#[proc_macro_derive(CliDispatch)]
pub fn cli_dispatch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let enum_name = &ast.ident;
    let reference_enum = if let syn::Data::Enum(e) = ast.data {
        e
    } else {
        panic!("CliDispatch can only be derived for Enums")
    };

    let branches = reference_enum.variants.iter().map(|v| {
        let name = &v.ident;
        if let Some(ext) = external_branch(v) {
            ext
        } else {
            quote! { Self::#name(cmd) => cmd.run() }
        }
    });

    let output = quote! {
        impl #enum_name {
            pub fn run(&self) -> anyhow::Result<(), anyhow::Error> {
                use durandal_core::CliCommand;

                match self {
                    #(#branches,)*
                }
            }
        }
    };

    proc_macro::TokenStream::from(output)
}

/// This macro reduces the boilerplate in dispatching cli subcommand executions.
///
/// Because this macro generates the dispatch logic for enum variants, it
/// naturally also only is valid for enums.
///
/// This macro _only_ works with the `CliMetaCommand` trait exposed by
/// `durandal-core`. If you need to dispatch to subcommands implementing
/// `CliCommand`, then use the [CliDispatch] macro instead.
#[proc_macro_derive(CliMetaDispatch, attributes(cli_meta))]
pub fn cli_meta_dispatch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let meta = if let Some(meta) = ast.attrs.iter().find(|a| a.path.is_ident("cli_meta")) {
        meta.parse_args::<TokenStream>().unwrap()
    } else {
        panic!("cli_meta attribute is required")
    };

    let enum_name = &ast.ident;
    let reference_enum = if let syn::Data::Enum(e) = ast.data {
        e
    } else {
        panic!("CliMetaDispatch can only be derived for Enums")
    };

    let branches = reference_enum.variants.iter().map(|v| {
        let name = &v.ident;
        if let Some(ext) = external_branch(v) {
            ext
        } else {
            quote! { Self::#name(cmd) => cmd.run(meta) }
        }
    });

    let output = quote! {
        impl #enum_name {
            pub fn run(&self, meta: &#meta) -> anyhow::Result<(), anyhow::Error> {
                use durandal_core::CliMetaCommand;

                match self {
                    #(#branches,)*
                }
            }
        }
    };

    proc_macro::TokenStream::from(output)
}
