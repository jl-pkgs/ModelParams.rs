use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit, Expr};

// ── shared helper ────────────────────────────────────────────────────────────

fn parse_f64_expr(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Lit(el) => match &el.lit {
            Lit::Float(f) => f.base10_parse().ok(),
            Lit::Int(i)   => i.base10_parse::<i64>().ok().map(|v| v as f64),
            _ => None,
        },
        Expr::Unary(u) if matches!(u.op, syn::UnOp::Neg(_)) =>
            parse_f64_expr(&u.expr).map(|v| -v),
        _ => None,
    }
}

struct FieldMeta {
    name:    syn::Ident,
    lo:      Option<f64>,
    hi:      Option<f64>,
    units:   String,
    default: Option<f64>,
    has_param: bool,   // has #[param] attr at all
    skip:    bool,     // #[param(skip)]
}

fn extract_fields(input: &DeriveInput) -> Vec<FieldMeta> {
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("only named fields are supported"),
        },
        _ => panic!("only structs are supported"),
    };

    fields.iter().map(|field| {
        let name = field.ident.clone().unwrap();
        let mut lo = None;
        let mut hi = None;
        let mut units = "-".to_string();
        let mut default = None;
        let mut skip = false;
        let mut has_param = false;

        if let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("param")) {
            has_param = true;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                } else if meta.path.is_ident("bounds") {
                    let value = meta.value()?;
                    let expr: Expr = value.parse()?;
                    if let Expr::Tuple(t) = expr {
                        if t.elems.len() == 2 {
                            lo = parse_f64_expr(&t.elems[0]);
                            hi = parse_f64_expr(&t.elems[1]);
                        }
                    }
                } else if meta.path.is_ident("units") {
                    let value = meta.value()?;
                    let lit: Lit = value.parse()?;
                    if let Lit::Str(s) = lit { units = s.value(); }
                } else if meta.path.is_ident("default") {
                    let value = meta.value()?;
                    let expr: Expr = value.parse()?;
                    default = parse_f64_expr(&expr);
                }
                Ok(())
            });
        }

        FieldMeta { name, lo, hi, units, default, has_param, skip }
    }).collect()
}

// ── #[derive(ModelParams)] ───────────────────────────────────────────────────

/// Generates `ModelParams` impl for a scalar parameter struct.
/// Use `#[param(bounds=(lo,hi), units="...", default=val)]` on each field.
/// Fields without `#[param]` are derived/fixed and not optimised.
/// Use `#[param(skip)]` to explicitly exclude a nested field from traversal.
#[proc_macro_derive(ModelParams, attributes(param))]
pub fn derive_model_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let fields = extract_fields(&input);

    let mut param_infos  = vec![];
    let mut get_values   = vec![];
    let mut set_values   = vec![];
    let mut idx = 0usize;

    for f in fields.iter().filter(|f| f.has_param && !f.skip) {
        let fname     = &f.name;
        let fname_str = fname.to_string();
        let units     = &f.units;
        let bounds_expr = match (f.lo, f.hi) {
            (Some(l), Some(h)) => quote! { Some((#l, #h)) },
            _                  => quote! { None },
        };
        let default_expr = f.default.map(|v| quote! { #v }).unwrap_or(quote! { 0.0_f64 });

        param_infos.push(quote! {
            modelparams::ParamInfo { name: #fname_str, bounds: #bounds_expr,
                                     units: #units, default: #default_expr }
        });
        get_values.push(quote! { self.#fname });
        set_values.push(quote! { #idx => self.#fname = value, });
        idx += 1;
    }

    let n = idx;
    quote! {
        impl modelparams::ModelParams for #name {
            fn param_info() -> Vec<modelparams::ParamInfo> { vec![ #(#param_infos),* ] }
            fn get_values(&self) -> Vec<f64> { vec![ #(#get_values),* ] }
            fn set_value(&mut self, index: usize, value: f64) {
                match index {
                    #(#set_values)*
                    _ => panic!("index {} out of range ({})", index, #n),
                }
            }
        }
    }.into()
}

// ── #[derive(Layers)] ────────────────────────────────────────────────────────

/// Generates a `{Name}Layers` struct where every `f64` field becomes `Vec<f64>`.
/// Also implements `NestedModelParams` for it (layer-indexed parameter traversal).
#[proc_macro_derive(Layers, attributes(param))]
pub fn derive_layers(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name  = &input.ident;
    let layers_name = format_ident!("{}Layers", name);
    let fields = extract_fields(&input);

    // only layer #[param]-annotated fields (skip Bool, Int, Vec fields)
    let param_fields: Vec<&FieldMeta> = fields.iter()
        .filter(|f| f.has_param && !f.skip)
        .collect();

    let struct_fields: Vec<TokenStream2> = param_fields.iter().map(|f| {
        let fname = &f.name;
        quote! { pub #fname: Vec<f64> }
    }).collect();

    let new_fields: Vec<TokenStream2> = param_fields.iter().map(|f| {
        let fname   = &f.name;
        let default = f.default.unwrap_or(0.0);
        quote! { #fname: vec![#default; n_layers] }
    }).collect();

    // collect_params: only #[param] fields with bounds, per layer index
    let collect_arms: Vec<TokenStream2> = fields.iter()
        .filter(|f| f.has_param && !f.skip && f.lo.is_some())
        .map(|f| {
            let fname     = &f.name;
            let fname_str = fname.to_string();
            let units     = &f.units;
            let lo        = f.lo.unwrap();
            let hi        = f.hi.unwrap();
            quote! {
                for i in 0..self.n_layers {
                    let mut path = prefix.to_vec();
                    path.push(#fname_str.to_string());
                    path.push(i.to_string());
                    params.push(modelparams::PathParam {
                        path,
                        name:   #fname_str.to_string(),
                        value:  self.#fname[i],
                        bounds: Some((#lo, #hi)),
                        units:  #units,
                    });
                }
            }
        }).collect();

    // update_param match arms
    let update_arms: Vec<TokenStream2> = fields.iter()
        .filter(|f| f.has_param && !f.skip && f.lo.is_some())
        .map(|f| {
            let fname     = &f.name;
            let fname_str = fname.to_string();
            quote! {
                #fname_str => {
                    if idx < self.#fname.len() { self.#fname[idx] = value; true }
                    else { false }
                }
            }
        }).collect();

    quote! {
        #[derive(Clone, Debug)]
        pub struct #layers_name {
            #(#struct_fields,)*
            pub n_layers: usize,
        }

        impl #layers_name {
            pub fn new(n_layers: usize) -> Self {
                Self { #(#new_fields,)* n_layers }
            }
        }

        impl modelparams::NestedModelParams for #layers_name {
            fn collect_params(&self, prefix: &[String]) -> Vec<modelparams::PathParam> {
                let mut params = Vec::new();
                #(#collect_arms)*
                params
            }

            fn update_param(&mut self, path: &[String], value: f64) -> bool {
                if path.len() < 2 { return false; }
                let idx: usize = match path[1].parse() {
                    Ok(i) => i, Err(_) => return false,
                };
                match path[0].as_str() {
                    #(#update_arms)*
                    _ => false,
                }
            }
        }
    }.into()
}
