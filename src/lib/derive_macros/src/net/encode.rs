use crate::helpers::{get_derive_attributes, StructInfo};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, LitInt, Fields};

// Helper function to extract packet ID from attributes
fn extract_packet_id(packet_attr: Vec<Attribute>) -> Option<u8> {
    let mut packet_id = None;
    packet_attr.iter().for_each(|attr| {
        attr.parse_nested_meta(|meta| {
            let Some(ident) = meta.path.get_ident() else {
                return Ok(());
            };

            if ident == "packet_id" {
                let value = meta.value().expect("value failed");
                let value = value.parse::<LitInt>().expect("parse failed");
                packet_id = Some(value.base10_parse::<u8>().expect("base10_parse failed"));
            }
            Ok(())
        }).unwrap();
    });
    packet_id
}

// Generate packet ID encoding snippets
fn generate_packet_id_snippets(packet_id: Option<u8>) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let sync_snippet = if let Some(id) = packet_id {
        quote! {
            <ferrumc_net_codec::net_types::var_int::VarInt as ferrumc_net_codec::encode::NetEncode>::encode(&#id.into(), writer, &ferrumc_net_codec::encode::NetEncodeOpts::None)?;
        }
    } else {
        quote! {}
    };

    let async_snippet = if let Some(id) = packet_id {
        quote! {
            <ferrumc_net_codec::net_types::var_int::VarInt as ferrumc_net_codec::encode::NetEncode>::encode_async(&#id.into(), writer, &ferrumc_net_codec::encode::NetEncodeOpts::None).await?;
        }
    } else {
        quote! {}
    };

    (sync_snippet, async_snippet)
}

// Generate field encoding expressions for structs
fn generate_field_encoders(fields: &syn::Fields) -> proc_macro2::TokenStream {
    let encode_fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        quote! {
            <#field_ty as ferrumc_net_codec::encode::NetEncode>::encode(&self.#field_name, writer, &ferrumc_net_codec::encode::NetEncodeOpts::None)?;
        }
    });
    quote! { #(#encode_fields)* }
}

fn generate_async_field_encoders(fields: &syn::Fields) -> proc_macro2::TokenStream {
    let encode_fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        quote! {
            <#field_ty as ferrumc_net_codec::encode::NetEncode>::encode_async(&self.#field_name, writer, &ferrumc_net_codec::encode::NetEncodeOpts::None).await?;
        }
    });
    quote! { #(#encode_fields)* }
}

// Generate enum variant encoding using static dispatch
fn generate_enum_encoders(data: &syn::DataEnum) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let variants = data.variants.iter().enumerate().map(|(idx, variant)| {
        let variant_ident = &variant.ident;
        let variant_idx = idx as u8;

        match &variant.fields {
            Fields::Named(fields) => {
                let field_idents: Vec<_> = fields.named.iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                let field_tys: Vec<_> = fields.named.iter()
                    .map(|f| &f.ty)
                    .collect();

                (quote! {
                    Self::#variant_ident { #(#field_idents),* } => {
                        #(
                            <#field_tys as ferrumc_net_codec::encode::NetEncode>::encode(#field_idents, writer, &ferrumc_net_codec::encode::NetEncodeOpts::None)?;
                        )*
                    }
                },
                 quote! {
                    Self::#variant_ident { #(#field_idents),* } => {
                        #(
                            <#field_tys as ferrumc_net_codec::encode::NetEncode>::encode_async(#field_idents, writer, &ferrumc_net_codec::encode::NetEncodeOpts::None).await?;
                        )*
                    }
                })
            },
            Fields::Unnamed(fields) => {
                let field_names: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| syn::Ident::new(&format!("field{}", i), proc_macro2::Span::call_site()))
                    .collect();
                let field_tys: Vec<_> = fields.unnamed.iter()
                    .map(|f| &f.ty)
                    .collect();

                (quote! {
                    Self::#variant_ident(#(#field_names),*) => {
                        <ferrumc_net_codec::net_types::var_int::VarInt as ferrumc_net_codec::encode::NetEncode>::encode(&#variant_idx.into(), writer, &ferrumc_net_codec::encode::NetEncodeOpts::None)?;
                        #(
                            <#field_tys as ferrumc_net_codec::encode::NetEncode>::encode(#field_names, writer, &ferrumc_net_codec::encode::NetEncodeOpts::None)?;
                        )*
                    }
                },
                 quote! {
                    Self::#variant_ident(#(#field_names),*) => {
                        <ferrumc_net_codec::net_types::var_int::VarInt as ferrumc_net_codec::encode::NetEncode>::encode_async(&#variant_idx.into(), writer, &ferrumc_net_codec::encode::NetEncodeOpts::None).await?;
                        #(
                            <#field_tys as ferrumc_net_codec::encode::NetEncode>::encode_async(#field_names, writer, &ferrumc_net_codec::encode::NetEncodeOpts::None).await?;
                        )*
                    }
                })
            },
            Fields::Unit => (
                quote! {
                    Self::#variant_ident => {
                        <ferrumc_net_codec::net_types::var_int::VarInt as ferrumc_net_codec::encode::NetEncode>::encode(&#variant_idx.into(), writer, &ferrumc_net_codec::encode::NetEncodeOpts::None)?;
                    }
                },
                quote! {
                    Self::#variant_ident => {
                        <ferrumc_net_codec::net_types::var_int::VarInt as ferrumc_net_codec::encode::NetEncode>::encode_async(&#variant_idx.into(), writer, &ferrumc_net_codec::encode::NetEncodeOpts::None).await?;
                    }
                }
            ),
        }
    }).unzip::<_, _, Vec<_>, Vec<_>>();

    let (sync_variants, async_variants) = variants;

    (
        quote! {
            match self {
                #(#sync_variants)*
            }
        },
        quote! {
            match self {
                #(#async_variants)*
            }
        }
    )
}
