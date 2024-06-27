extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, ImplItem, FnArg, ReturnType, ImplItemMethod, GenericArgument, PathArguments, Item, ItemEnum,
    ItemImpl, ItemMod, token::Brace, ItemStruct, Type, Visibility, TypePath, AngleBracketedGenericArguments
};


#[proc_macro_derive(WithNew)]
pub fn with_new(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let generic_params: Vec<_> = generics.params.iter().collect();
    // `where_clause` for future traits bounds handling
    let where_clause = &generics.where_clause;

    let gen = match &input.data {
        Data::Struct(data) => {
            // Extract the list of field names and types
            let (field_names, field_types): (Vec<_>, Vec<_>) = match &data.fields {
                Fields::Named(ref fields_named) => fields_named
                    .named
                    .iter()
                    .map(|f| (f.ident.clone(), &f.ty))
                    .unzip(),
                _ => panic!("AutoPyClass can only be used with structs with named fields"),
            };

            // Partition fields into required and optional
            let (required_fields, optional_fields): (Vec<_>, Vec<_>) = field_names.iter()
            .zip(field_types.iter())
            .partition(|(_, ty)| !matches!(ty, Type::Path(type_path) if type_path.path.segments.iter().any(|segment| segment.ident == "Option")));

            let required_field_names: Vec<_> =
                required_fields.iter().map(|(name, _)| name).collect();
            let required_field_types: Vec<_> = required_fields.iter().map(|(_, ty)| ty).collect();

            let optional_field_names: Vec<_> =
                optional_fields.iter().map(|(name, _)| name).collect();
            let optional_field_types: Vec<_> = optional_fields.iter().map(|(_, ty)| ty).collect();

            let all_values = quote! { #(#field_names),*};

            let required_part = if !required_field_names.is_empty() {
                quote! { #(#required_field_names: #required_field_types),* }
            } else {
                quote! {}
            };
            let optional_part = if !optional_field_names.is_empty() {
                quote! { #(#optional_field_names: #optional_field_types),* }
            } else {
                quote! {}
            };

            let combined_arguments =
                if !required_field_names.is_empty() && !optional_field_names.is_empty() {
                    quote! { #required_part, #optional_part }
                } else {
                    quote! { #required_part #optional_part }
                };
            let combined_signatures =
                if !required_field_names.is_empty() && !optional_field_names.is_empty() {
                    quote! { #(#required_field_names),*, #(#optional_field_names = None),* }
                } else if !required_field_names.is_empty() {
                    quote! { #(#required_field_names),* }
                } else {
                    quote! { #(#optional_field_names = None),* }
                };

            if generic_params.is_empty() {
                // Implement methods template of the `new()` function
                quote! {                    
                    #[pymethods]
                    impl #name {
                        // By default, it is not possible to create an instance of a custom class from Python code.
                        // To declare a constructor, you need to define a method and annotate it with the #[new] attribute.
                        // https://pyo3.rs/v0.21.2/class#constructor

                        // Most arguments are required by default, except for trailing Option<_> arguments, which are implicitly given a default of None.
                        // This behaviour can be configured by the #[pyo3(signature = (...))] option which allows writing a signature in Python syntax.
                        // https://pyo3.rs/v0.21.2/function/signature#trailing-optional-arguments
                        #[new]
                        #[pyo3(signature = ( #combined_signatures ) )]
                        pub fn new(#combined_arguments) -> Self {
                            Self {
                                #all_values
                            }
                        }

                        // use prost::Message;
                        // use pyo3::types::PyBytes;
                        pub fn ParseFromString(&mut self, bytes_string: &pyo3::types::PyBytes) -> Result<#name, crate::flyteidl::MessageDecodeError> {
                            let bt = bytes_string.as_bytes();
                            let de = prost::Message::decode(&bt.to_vec()[..]);
                            Ok(de?)
                        }
                        // TODO: 
                        // pub fn SerializeToString(proto_obj: #name) -> Result<Vec<u8>, crate::flyteidl::MessageEncodeError> {
                        //     let mut buf = vec![];
                        //     proto_obj.encode(&mut buf)?;
                        //     Ok(buf)
                        // }
                    }


                    // https://github.com/hyperium/tonic/blob/c7836521dd417434d625bd653fcf00fb7f7ae25e/tonic/src/request.rs#L28
                }
            } else {
                // https://pyo3.rs/v0.21.2/class#no-generic-parameters
                // TODO: Implement methods template of the `new()` function for generic structs
                quote! {}
            }
        }
        // Data::Struct(data) => {
        //     // Extract the list of field names and types
        //     let (field_names, field_types): (Vec<_>, Vec<_>) = match &data.fields {
        //         Fields::Named(ref fields_named) => fields_named
        //             .named
        //             .iter()
        //             .map(|f| (f.ident.clone(), &f.ty))
        //             .unzip(),
        //         _ => panic!("AutoPyClass can only be used with structs with named fields"),
        //     };
        
        //     let all_values = quote! { #(#field_names),*};
        
        //     // Make all fields optional
        //     let optional_field_names: Vec<_> = field_names.iter().collect();
        //     let optional_field_types: Vec<_> = field_types.iter().map(|ty| quote! { #ty }).collect();
        
        //     let optional_part = if !optional_field_names.is_empty() {
        //         quote! { #(#optional_field_names: #optional_field_types),* }
        //     } else {
        //         quote! {}
        //     };
        
        //     let combined_signatures = if !optional_field_names.is_empty() {
        //         quote! { #(#optional_field_names = None),* }
        //     } else {
        //         quote! {}
        //     };
        
        //     if generic_params.is_empty() {
        //         // Implement methods template of the `new()` function
        //         quote! {
        //             #[::pyo3::pymethods]
        //             impl #name {
        //                 #[new]
        //                 #[pyo3(signature = ( #combined_signatures ) )]
        //                 pub fn new(#optional_part) -> Self {
        //                     Self {
        //                         #all_values
        //                     }
        //                 }
        //             }
        //         }
        //     } else {
        //         // TODO: Implement methods template of the `new()` function for generic structs
        //         quote! {}
        //     }
        // }
        Data::Enum(_data) => {
            quote! {}
        }
        Data::Union(_data) => {
            quote! {}
        }
        _ => panic!("AutoPyClass can only be used with structs enums and unions"),
    };

    TokenStream::from(gen)
}

#[proc_macro_attribute]
pub fn with_pyclass(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Item);

    // Reconstruct the struct or enum definition block
    let output = match input {
        Item::Struct(item_struct) => {
            quote! {
                use pyo3::prelude::*;
                #[pyclass(subclass, dict, get_all, set_all)]
                #item_struct
            }
        }
        Item::Enum(item_enum) => {
            quote! {
                use pyo3::prelude::*;
                #[pyclass(get_all, set_all)]
                #item_enum
            }
        }
        _ => {
            return syn::Error::new_spanned(
                input,
                "with_pyclass can only be used with structs or enums",
            )
            .to_compile_error()
            .into();
        }
    };

    output.into()
}


// #[proc_macro_attribute]
// pub fn with_string(_: TokenStream, input: TokenStream) -> TokenStream {
//     // Parse the input tokens into a syntax tree
//     let input = parse_macro_input!(input as Item);

//     // Check if the input is a module
//     if let Item::Mod(item_mod) = input {
//         let mod_name = &item_mod.ident;
//         let items = &item_mod.content;

//         let output = if let Some((brace, items)) = items {
//             quote! {
//                 pub mod #mod_name {
//                     use core::str;
//                     use std::fmt;
//                     pub use pyo3::prelude::*;
//                     use pyo3::PyErr;
//                     use pyo3::exceptions::PyOSError;
//                     use pyo3::types::{PyBytes};
//                     use tonic::Status;
//                     use prost::{DecodeError, EncodeError, Message};
                    
                    
//                     // An error indicates taht failing at serializing object to bytes string, like `SerializTOString()` for python protos.
//                     pub struct MessageEncodeError(EncodeError);
//                     // An error indicates taht failing at deserializing object from bytes string, like `ParseFromString()` for python protos.
//                     pub struct MessageDecodeError(DecodeError);

//                     impl fmt::Display for MessageEncodeError {
//                         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//                             write!(f, "")
//                         }
//                     }

//                     impl fmt::Display for MessageDecodeError {
//                         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//                             write!(f, "")
//                         }
//                     }

//                     impl std::convert::From<MessageEncodeError> for PyErr {
//                         fn from(err: MessageEncodeError) -> PyErr {
//                             PyOSError::new_err(err.to_string())
//                         }
//                     }

//                     impl std::convert::From<MessageDecodeError> for PyErr {
//                         fn from(err: MessageDecodeError) -> PyErr {
//                             PyOSError::new_err(err.to_string())
//                         }
//                     }

//                     impl std::convert::From<EncodeError> for MessageEncodeError {
//                         fn from(other: EncodeError) -> Self {
//                             Self(other)
//                         }
//                     }

//                     impl std::convert::From<DecodeError> for MessageDecodeError {
//                         fn from(other: DecodeError) -> Self {
//                             Self(other)
//                         }
//                     }

//                     pub trait ProtobufDecoder<T> where T: Message + Default {
//                         fn decode_proto(&self, bytes_obj: &PyBytes) -> Result<T, MessageDecodeError>;
//                     }
                    
//                     pub trait ProtobufEncoder<T> where T: Message + Default {
//                         fn encode_proto(&self, res: T) -> Result<Vec<u8>, MessageEncodeError>;
//                     }
                    
//                     impl<T> ProtobufDecoder<T> for T where T: Message + Default {
//                         fn decode_proto(&self, bytes_obj: &PyBytes) -> Result<T, MessageDecodeError> {
//                             let bytes = bytes_obj.as_bytes();
//                             let de = Message::decode(&bytes.to_vec()[..]);
//                             Ok(de?)
//                         }
//                     }
                    
//                     impl<T> ProtobufEncoder<T> for T where T: Message + Default {
//                         fn encode_proto(&self, res: T) -> Result<Vec<u8>, MessageEncodeError> {
//                             let mut buf = vec![];
//                             res.encode(&mut buf)?;
//                             Ok(buf)
//                         }
//                     }

//                     use pyo3::prelude::*;
//                     #(#items)*
//                 }
//             }
//         } else {
//             // If the module has no content (an external module declaration)
//             quote! {
//                 pub mod #mod_name;

//                 use core::str;
//                     use std::fmt;
//                     pub use pyo3::prelude::*;
//                     use pyo3::PyErr;
//                     use pyo3::exceptions::PyOSError;
//                     use pyo3::types::{PyBytes};
//                     use tonic::Status;
//                     use prost::{DecodeError, EncodeError, Message};
                    
                    
//                     // An error indicates taht failing at serializing object to bytes string, like `SerializTOString()` for python protos.
//                     pub struct MessageEncodeError(EncodeError);
//                     // An error indicates taht failing at deserializing object from bytes string, like `ParseFromString()` for python protos.
//                     pub struct MessageDecodeError(DecodeError);

//                     impl fmt::Display for MessageEncodeError {
//                         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//                             write!(f, "")
//                         }
//                     }

//                     impl fmt::Display for MessageDecodeError {
//                         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//                             write!(f, "")
//                         }
//                     }

//                     impl std::convert::From<MessageEncodeError> for PyErr {
//                         fn from(err: MessageEncodeError) -> PyErr {
//                             PyOSError::new_err(err.to_string())
//                         }
//                     }

//                     impl std::convert::From<MessageDecodeError> for PyErr {
//                         fn from(err: MessageDecodeError) -> PyErr {
//                             PyOSError::new_err(err.to_string())
//                         }
//                     }

//                     impl std::convert::From<EncodeError> for MessageEncodeError {
//                         fn from(other: EncodeError) -> Self {
//                             Self(other)
//                         }
//                     }

//                     impl std::convert::From<DecodeError> for MessageDecodeError {
//                         fn from(other: DecodeError) -> Self {
//                             Self(other)
//                         }
//                     }

//                     pub trait ProtobufDecoder<T> where T: Message + Default {
//                         fn decode_proto(&self, bytes_obj: &PyBytes) -> Result<T, MessageDecodeError>;
//                     }
                    
//                     pub trait ProtobufEncoder<T> where T: Message + Default {
//                         fn encode_proto(&self, res: T) -> Result<Vec<u8>, MessageEncodeError>;
//                     }
                    
//                     impl<T> ProtobufDecoder<T> for T where T: Message + Default {
//                         fn decode_proto(&self, bytes_obj: &PyBytes) -> Result<T, MessageDecodeError> {
//                             let bytes = bytes_obj.as_bytes();
//                             let de = Message::decode(&bytes.to_vec()[..]);
//                             Ok(de?)
//                         }
//                     }
                    
//                     impl<T> ProtobufEncoder<T> for T where T: Message + Default {
//                         fn encode_proto(&self, res: T) -> Result<Vec<u8>, MessageEncodeError> {
//                             let mut buf = vec![];
//                             res.encode(&mut buf)?;
//                             Ok(buf)
//                         }
//                     }

//             }
//         };

//         output.into()
//     } else {
//         // If the input is not a module, return it unchanged
//         quote! { #input }.into()
//     }
// }



// #[proc_macro_attribute]
// pub fn list_all_async_methods(_: TokenStream, input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as ItemMod);
//     let mod_name = &input.ident;
//     let mut method_tuples = Vec::new();
//     let mut struct_name = None;
//     let mut items = vec![];
//     let mut impl_blocks = vec![];

//     // Extract struct name and async methods
//     for item in input.content.unwrap().1 {
//         if let Item::Struct(ItemStruct { ident, .. }) = &item {
//             struct_name = Some(ident.clone());
//         }

//         if let Item::Impl(item_impl) = &item {
//             if let Type::Path(type_path) = &*item_impl.self_ty {
//                 if struct_name.is_none() {
//                     struct_name = Some(type_path.path.segments.last().unwrap().ident.clone());
//                 }
//                 for impl_item in &item_impl.items {
//                     if let ImplItem::Method(method) = impl_item {
//                         if method.sig.asyncness.is_some() {
//                             let method_name = method.sig.ident.clone();
//                             let mut inputs = Vec::new();

//                             // Collect input types
//                             for input in &method.sig.inputs {
//                                 if let FnArg::Typed(pat_type) = input {
//                                     inputs.push(deepest_type(&pat_type.ty));
//                                 }
//                             }

//                             // Collect return type
//                             let output = match &method.sig.output {
//                                 ReturnType::Default => quote! {()},
//                                 ReturnType::Type(_, ty) => deepest_type(&ty),
//                             };

//                             let inputs: proc_macro2::TokenStream = quote! { (#(#inputs),*) };
//                             method_tuples.push((method_name, inputs, output));
//                         }
//                     }
//                 }
//                 impl_blocks.push(item_impl.clone());
//                 continue;
//             }
//         }

//         items.push(item);
//     }

//     let struct_name = struct_name.expect("No struct found in the module.");

//     // Generate list_all_async_methods function
//     let list_all_async_methods_fn = {
//         let method_names = method_tuples.iter().map(|(name, _, _)| name);
//         let input_types = method_tuples.iter().map(|(_, inputs, _)| inputs);
//         let output_types = method_tuples.iter().map(|(_, _, output)| output);
//         quote! {
//             pub fn list_all_async_methods() -> Vec<(&'static str, &'static str, &'static str)> {
//                 vec![
//                     #((stringify!(#method_names), stringify!(#input_types), stringify!(#output_types))),*
//                 ]
//             }
//         }
//     };

//     // Add list_all_async_methods function to the last impl block
//     if let Some(last_impl) = impl_blocks.last_mut() {
//         last_impl.items.push(ImplItem::Method(syn::parse_quote! {
//             #list_all_async_methods_fn
//         }));
//     }

//     // Reassemble the module with updated impl blocks
//     items.extend(impl_blocks.into_iter().map(Item::Impl));

//     let output = quote! {
//         pub mod #mod_name {
//             #(#items)*
//         }
//     };

//     output.into()
// }

// fn deepest_type(ty: &Type) -> proc_macro2::TokenStream {
//     match ty {
//         Type::Path(type_path) => {
//             if let Some(segment) = type_path.path.segments.last() {
//                 if segment.arguments.is_empty() {
//                     return quote! {#type_path};
//                 } else {
//                     if let PathArguments::AngleBracketed(args) = &segment.arguments {
//                         if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
//                             return deepest_type(inner_ty);
//                         }
//                     }
//                 }
//             }
//         }
//         _ => {}
//     }
//     quote! {#ty}
// }