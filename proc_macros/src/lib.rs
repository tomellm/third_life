extern crate proc_macro;


use core::panic;

use proc_macro::{TokenStream, Span};
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, PathArguments, GenericArgument, Path, Type};


/// This derive macro is intended to be used in combination with the 
/// [`third_life::config::ConfigurationLoader`] trait. When used on a struct
/// that implements that trait it will create a new struct of the same name with
/// a postfix of `Plugin` and implement the [`bevy::prelude::Plugin`] trait for 
/// it. The resulting Plugin struct still needs to be manually added to the
/// application
///
/// ```
/// #[derive(ConfigFile)]
/// struct MedicineConfig {
///     medicine_level: f32
/// }
/// 
/// // Assuming that [`third_life::config::ConfigurationLoader`] is implemented
/// // for this struct the this is thes code the macro will create:
///
/// struct MedicineConfigPlugin;
///
/// impl Plugin for MedicineConfigPlugin {
///     fn build(&self, app: &mut App) {
///         MedicineConfig::add_configuration(app);
///     }
/// }
/// ```
///
/// Continue reading the [`proc_macros::Config`] for further configuration
/// setup.
#[proc_macro_derive(ConfigFile)]
pub fn derive_configuration_plugin(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let conf_name = input.ident.clone();
    let plugin_name = format_ident!("{conf_name}Plugin");
    let gen = quote! {
        pub struct #plugin_name;
        impl Plugin for #plugin_name {
            fn build(&self, app: &mut App) {
                #conf_name::add_configuration(app);
            }
        }
    };
    gen.into()
}



// https://github.com/emk/accessors/blob/master/src/lib.rs
// https://stackoverflow.com/questions/55271857/how-can-i-get-the-t-from-an-optiont-when-using-syn


/// The Config provides automatic getters with defaults for structs. Any struct
/// on which you add the config macros will have getters created for it. These 
/// getters [`Clone::clone`] values out of the struct and unwrap any options. 
///
/// To provide defaults add a attribute level macro on top of the target field
/// with the default value inside.
/// ```
/// #[derive(Config)]
/// pub struct PopulationConfig {
///     size: f32,
///     #[def(30.)]
///     median: Option<f32>,
/// }
/// // This will create a implementation with getters like so:
/// impl PopulationConfig {
///     pub fn size(&self) -> f32 {
///         self.clone()
///     }
///     pub fn median(&self) -> f32 {
///         self.clone().unwrap_or((30.)) // I dont know how the get rid of the extra brackets
///     }
/// }
/// ```
/// `IMPORTANT` Using this macro creates a implementation for the struct meaning
/// no other implementaion can be created. This is fine in my opinion since its
/// only configuration but is still to be considered.
#[proc_macro_derive(Config, attributes(def))]
pub fn derive_getters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let expanded = expand_getters(ast);
    expanded.to_string().parse().unwrap()
}

fn expand_getters(mut ast: DeriveInput) -> TokenStream {
    // println!("Defining getters for: {:#?}", ast);

    //extract_attrs(&mut ast.attrs, "getters");
    
    enum FieldType<'a> {
        Option(&'a Type),
        NotOption(&'a Type)
    }

    let name = &ast.ident;
    
    let fields: Vec<_> = match ast.data {
        Data::Struct(ref data) => {
            data.fields.iter().map(|f| {
                let ty = match extract_type_from_option(&f.ty) {
                    Some(t) => FieldType::Option(t),
                    None => FieldType::NotOption(&f.ty)
                };
                let mut def = None;
                for attr in f.attrs.iter() {
                    if attr.path.is_ident("def") {
                        def = Some(&attr.tokens);
                    }
                }
                (f.ident.as_ref().unwrap(), ty, def)
            }).collect()
        },
        _ => panic!("#[derive(Config)] can only be used on braced structs")
    };


    let quotes: Vec<_> = fields.iter().map(|(field, ty, def)| {
        match (ty, def) {
            (FieldType::Option(ty), Some(def)) => quote! {
                pub fn #field(&self) -> #ty {
                    self.#field.clone().unwrap_or(#def)
                }
            },
            (FieldType::NotOption(ty), _) => quote!{
                pub fn #field(&self) -> #ty {
                    self.#field.clone()
                }
            },
            _ => panic!("If a value is an option there has to be default value for it!")
        }
    }).collect();


    let def_fn = if fields.iter().all(|e|e.2.is_some()) {
        let (idents, defaults) = fields.into_iter()
            .fold(
                (Vec::new(), Vec::new()), |(mut acc1, mut acc2), (i, f, d)| {
                    acc1.push(i);
                    let u_d = d.unwrap();
                    acc2.push(match f {
                        FieldType::Option(_) => quote!{ Some(#u_d) },
                        FieldType::NotOption(_) => quote!{ #u_d }
                    });
                    (acc1, acc2)
                }
            );
        Some(quote!{
            pub fn def_conf() -> Self {
                Self {
                    #(#idents: #defaults,)*
                }
            }
        })
    } else {
        None
    };

    quote! {
        impl #name {

            #def_fn

            #(#quotes)*
        }
    }.into()
}

fn extract_type_from_option(ty: &Type) -> Option<&Type> {
    fn path_is_option(path: &Path) -> bool {
        path.leading_colon.is_none()
            && path.segments.len() == 1
            && path.segments.iter().next().unwrap().ident == "Option"
    }

    match ty {
        Type::Path(typepath) if typepath.qself.is_none() && path_is_option(&typepath.path) => {
            // Get the first segment of the path (there is only one, in fact: "Option"):
            let type_params = &typepath.path.segments.first().unwrap().arguments;
            // It should have only on angle-bracketed param ("<String>"):
            let generic_arg = match type_params {
                PathArguments::AngleBracketed(params) => Some(params.args.first().unwrap()),
                _ => None,
            }?;
            // This argument must be a type:
            match generic_arg {
                GenericArgument::Type(ty) => Some(ty),
                _ => None,
            }
        }
        _ => None,
    }
}
