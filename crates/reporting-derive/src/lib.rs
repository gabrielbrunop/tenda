use proc_macro::TokenStream;
use quote::quote;
use std::collections::BTreeMap;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, LitStr, Type};

/// Derive macro for creating a `Report` implementation that supports:
///
/// - `#[span]` for the primary span
/// - `#[label]` for extra spans (Option, SourceSpan, Vec<SourceSpan>)
/// - `#[help]`/`#[note]` for textual messages (Option<String>, Vec<String>)
/// - `#[message]` for overriding the default error message
/// - `#[metadata]` for fields you want get/set methods generated for
/// - `#[report("error_kind")]` to customize the error kind
/// - `#[accept_hooks]` so you provide your own `HasDiagnosticHooks` impl;
///   otherwise a default empty `HasDiagnosticHooks` is given.
#[proc_macro_derive(
    Diagnostic,
    attributes(report, span, label, help, note, message, metadata, accept_hooks)
)]
pub fn report_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let accept_hooks = input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("accept_hooks"));

    let error_kind = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("report"))
        .and_then(|attr| attr.parse_args::<LitStr>().ok())
        .map(|lit| lit.value())
        .unwrap_or_else(|| "erro".into());

    let enum_ident = &input.ident;

    let mut get_span_arms = Vec::new();
    let mut set_span_arms = Vec::new();
    let mut get_label_arms = Vec::new();
    let mut get_help_arms = Vec::new();
    let mut get_note_arms = Vec::new();
    let mut get_message_arms = Vec::new();

    #[derive(Default)]
    struct MetaFieldInfo {
        field_type: Option<Type>,
        get_arms: Vec<proc_macro2::TokenStream>,
        get_mut_arms: Vec<proc_macro2::TokenStream>,
        set_arms: Vec<proc_macro2::TokenStream>,
    }

    let mut metadata_map: BTreeMap<Ident, MetaFieldInfo> = BTreeMap::new();

    let enum_data = match &input.data {
        Data::Enum(data_enum) => data_enum,
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "`Report` can only be derived for enums.",
            )
            .to_compile_error()
            .into();
        }
    };

    for variant in &enum_data.variants {
        let variant_ident = &variant.ident;

        let mut get_span_arm = quote! { Self::#variant_ident { .. } => None };
        let mut set_span_arm = quote! { Self::#variant_ident { .. } => {} };
        let mut get_label_arm = quote! { Self::#variant_ident { .. } => Vec::new() };
        let mut get_help_arm = quote! { Self::#variant_ident { .. } => Vec::new() };
        let mut get_note_arm = quote! { Self::#variant_ident { .. } => Vec::new() };
        let mut get_message_arm = quote! { Self::#variant_ident { .. } => None };

        if let Fields::Named(named_fields) = &variant.fields {
            let mut span_fields_found = 0;
            let mut message_fields_found = 0;

            let mut span_fields = Vec::new();
            let mut label_fields = Vec::new();
            let mut help_fields = Vec::new();
            let mut note_fields = Vec::new();
            let mut message_fields = Vec::new();

            for field in &named_fields.named {
                let field_ident = field.ident.as_ref().unwrap();
                let field_ty = &field.ty;

                let mut is_span = false;
                let mut is_label = false;
                let mut is_help = false;
                let mut is_note = false;
                let mut is_message = false;
                let mut is_metadata = false;

                for attr in &field.attrs {
                    if attr.path().is_ident("span") {
                        is_span = true;
                    } else if attr.path().is_ident("label") {
                        is_label = true;
                    } else if attr.path().is_ident("help") {
                        is_help = true;
                    } else if attr.path().is_ident("note") {
                        is_note = true;
                    } else if attr.path().is_ident("message") {
                        is_message = true;
                    } else if attr.path().is_ident("metadata") {
                        is_metadata = true;
                    }
                }

                if is_span {
                    span_fields_found += 1;
                    if span_fields_found > 1 {
                        return syn::Error::new_spanned(
                            field,
                            format!(
                                "Multiple `#[span]` attributes in variant `{}` are not allowed.",
                                variant_ident
                            ),
                        )
                        .to_compile_error()
                        .into();
                    }
                    if let Type::Path(type_path) = &field.ty {
                        if let Some(last_seg) = type_path.path.segments.last() {
                            match last_seg.ident.to_string().as_str() {
                                "Option" => {
                                    span_fields.push((
                                        field_ident.clone(),
                                        quote! {
                                            Self::#variant_ident { #field_ident, .. } => #field_ident.clone()
                                        },
                                        quote! {
                                            Self::#variant_ident { #field_ident, .. } => {
                                                *#field_ident = Some(new_span.clone());
                                            }
                                        }
                                    ));
                                }
                                "SourceSpan" => {
                                    span_fields.push((
                                        field_ident.clone(),
                                        quote! {
                                            Self::#variant_ident { #field_ident, .. } => Some(#field_ident.clone())
                                        },
                                        quote! {
                                            Self::#variant_ident { #field_ident, .. } => {
                                                *#field_ident = new_span.clone();
                                            }
                                        }
                                    ));
                                }
                                _ => {
                                    return syn::Error::new_spanned(
                                        &field.ty,
                                        "Expected `#[span]` field to be `SourceSpan` or `Option<SourceSpan>`",
                                    ).to_compile_error().into();
                                }
                            }
                        }
                    }
                }

                if is_label {
                    if let Type::Path(type_path) = &field.ty {
                        if let Some(last_seg) = type_path.path.segments.last() {
                            match last_seg.ident.to_string().as_str() {
                                "Option" => label_fields
                                    .push((field_ident.clone(), "OptionSpan".to_string())),
                                "Vec" => {
                                    label_fields.push((field_ident.clone(), "VecSpan".to_string()))
                                }
                                "SourceSpan" => label_fields
                                    .push((field_ident.clone(), "DirectSpan".to_string())),
                                _ => {
                                    return syn::Error::new_spanned(
                                        &field.ty,
                                        "Expected `#[label]` field to be `SourceSpan`, `Option<SourceSpan>`, or `Vec<SourceSpan>`",
                                    ).to_compile_error().into();
                                }
                            }
                        }
                    }
                }

                let check_help_note = |ty: &Type| {
                    if let Type::Path(tp) = ty {
                        if let Some(seg) = tp.path.segments.last() {
                            match seg.ident.to_string().as_str() {
                                "Option" => Ok("OptionString".to_string()),
                                "Vec" => Ok("VecString".to_string()),
                                _ => Err(syn::Error::new_spanned(
                                    tp,
                                    "Expected field to be `Option<String>` or `Vec<String>`",
                                )),
                            }
                        } else {
                            Err(syn::Error::new_spanned(
                                tp,
                                "Expected field to be `Option<String>` or `Vec<String>`",
                            ))
                        }
                    } else {
                        Err(syn::Error::new_spanned(
                            ty,
                            "Expected field to be `Option<String>` or `Vec<String>`",
                        ))
                    }
                };

                if is_help {
                    match check_help_note(field_ty) {
                        Ok(kind) => help_fields.push((field_ident.clone(), kind)),
                        Err(e) => return e.to_compile_error().into(),
                    }
                }

                if is_note {
                    match check_help_note(field_ty) {
                        Ok(kind) => note_fields.push((field_ident.clone(), kind)),
                        Err(e) => return e.to_compile_error().into(),
                    }
                }

                if is_message {
                    message_fields_found += 1;
                    if message_fields_found > 1 {
                        return syn::Error::new_spanned(
                            field,
                            format!(
                                "Multiple `#[message]` attributes in variant `{}` are not allowed.",
                                variant_ident
                            ),
                        )
                        .to_compile_error()
                        .into();
                    }
                    if let Type::Path(type_path) = &field.ty {
                        if let Some(last_seg) = type_path.path.segments.last() {
                            match last_seg.ident.to_string().as_str() {
                                "String" => {
                                    message_fields
                                        .push((field_ident.clone(), "DirectString".to_string()));
                                }
                                "Option" => {
                                    message_fields
                                        .push((field_ident.clone(), "OptionString".to_string()));
                                }
                                _ => {
                                    return syn::Error::new_spanned(
                                        &field.ty,
                                        "Expected `#[message]` field to be `String` or `Option<String>`"
                                    ).to_compile_error().into();
                                }
                            }
                        }
                    }
                }

                if is_metadata {
                    let entry = metadata_map.entry(field_ident.clone()).or_default();
                    if entry.field_type.is_none() {
                        entry.field_type = Some(field_ty.clone());
                    }
                    let get_arm = quote! {
                        Self::#variant_ident { #field_ident, .. } => Some(#field_ident),
                    };
                    let get_mut_arm = quote! {
                        Self::#variant_ident { ref mut #field_ident, .. } => Some(#field_ident),
                    };
                    let set_arm = quote! {
                        Self::#variant_ident { #field_ident, .. } => { *#field_ident = value; },
                    };
                    entry.get_arms.push(get_arm);
                    entry.get_mut_arms.push(get_mut_arm);
                    entry.set_arms.push(set_arm);
                }
            }

            if let Some((_, get_code, set_code)) = span_fields.last() {
                get_span_arm = get_code.clone();
                set_span_arm = set_code.clone();
            }

            if !label_fields.is_empty() {
                let all_idents: Vec<_> = named_fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();

                let gather_label_branches: Vec<_> = label_fields
                    .iter()
                    .map(|(ident, kind)| match kind.as_str() {
                        "OptionSpan" => quote! {
                            if let Some(s) = #ident.clone() {
                                labels.push(s);
                            }
                        },
                        "VecSpan" => quote! {
                            labels.extend(#ident.clone());
                        },
                        "DirectSpan" => quote! {
                            labels.push(#ident.clone());
                        },
                        _ => unreachable!(),
                    })
                    .collect();

                let label_arm = quote! {
                    Self::#variant_ident { #(ref #all_idents),* } => {
                        let mut labels = Vec::new();
                        #( #gather_label_branches )*
                        labels
                    }
                };
                get_label_arm = label_arm;
            }

            if !help_fields.is_empty() {
                let all_idents: Vec<_> = named_fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();

                let pushes: Vec<_> = help_fields
                    .iter()
                    .map(|(ident, kind)| {
                        if kind == "OptionString" {
                            quote! {
                                if let Some(val) = #ident.clone() {
                                    helps.push(val);
                                }
                            }
                        } else {
                            quote! {
                                helps.extend(#ident.clone());
                            }
                        }
                    })
                    .collect();

                let help_arm = quote! {
                    Self::#variant_ident { #(ref #all_idents),* } => {
                        let mut helps = Vec::new();
                        #( #pushes )*
                        helps
                    }
                };
                get_help_arm = help_arm;
            }

            if !note_fields.is_empty() {
                let all_idents: Vec<_> = named_fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();

                let pushes: Vec<_> = note_fields
                    .iter()
                    .map(|(ident, kind)| {
                        if kind == "OptionString" {
                            quote! {
                                if let Some(val) = #ident.clone() {
                                    notes.push(val);
                                }
                            }
                        } else {
                            quote! {
                                notes.extend(#ident.clone());
                            }
                        }
                    })
                    .collect();

                let note_arm = quote! {
                    Self::#variant_ident { #(ref #all_idents),* } => {
                        let mut notes = Vec::new();
                        #( #pushes )*
                        notes
                    }
                };
                get_note_arm = note_arm;
            }

            if let Some((ident, kind)) = message_fields.last() {
                let message_arm = match kind.as_str() {
                    "DirectString" => quote! {
                        Self::#variant_ident { #ident, .. } => {
                            Some(#ident.clone())
                        }
                    },
                    "OptionString" => quote! {
                        Self::#variant_ident { #ident, .. } => {
                            #ident.clone()
                        }
                    },
                    _ => quote! {
                        Self::#variant_ident { .. } => None
                    },
                };
                get_message_arm = message_arm;
            }
        }

        get_span_arms.push(get_span_arm);
        set_span_arms.push(set_span_arm);
        get_label_arms.push(get_label_arm);
        get_help_arms.push(get_help_arm);
        get_note_arms.push(get_note_arm);
        get_message_arms.push(get_message_arm);
    }

    let report_impl = quote! {
        impl tenda_reporting::Diagnostic<tenda_common::span::SourceSpan> for #enum_ident {
            fn get_span(&self) -> ::std::option::Option<tenda_common::span::SourceSpan> {
                match self {
                    #( #get_span_arms ),*
                }
            }

            fn set_span(&mut self, new_span: &tenda_common::span::SourceSpan) {
                match self {
                    #( #set_span_arms ),*
                }
            }

            fn get_labels(&self) -> ::std::vec::Vec<tenda_common::span::SourceSpan> {
                match self {
                    #( #get_label_arms ),*
                }
            }

            fn get_helps(&self) -> ::std::vec::Vec<String> {
                match self {
                    #( #get_help_arms ),*
                }
            }

            fn get_notes(&self) -> ::std::vec::Vec<String> {
                match self {
                    #( #get_note_arms ),*
                }
            }

            fn get_message(&self) -> ::std::option::Option<String> {
                match self {
                    #( #get_message_arms ),*
                }
            }

            fn build_report_config(&self) -> tenda_reporting::DiagnosticConfig<tenda_common::span::SourceSpan> {
                let fallback_message = self.to_string();
                let span = self.get_span();
                let labels = self.get_labels();
                let helps = self.get_helps();
                let notes = self.get_notes();
                let final_message = if let Some(custom) = self.get_message() {
                    custom
                } else {
                    fallback_message
                };
                let stacktrace = vec![];

                tenda_reporting::DiagnosticConfig::new(
                    span,
                    labels,
                    helps,
                    notes,
                    final_message,
                    stacktrace,
                )
            }

            fn to_report(&self) -> tenda_reporting::Report<tenda_common::span::SourceSpan> {
                use tenda_reporting::Fmt;
                use tenda_reporting::{HasDiagnosticHooks, DiagnosticConfig};

                let kind = tenda_reporting::ReportKind::Custom(&#error_kind, tenda_reporting::Color::Red);
                let prefixes = tenda_reporting::Localization::new()
                    .with_help("ajuda")
                    .with_note("nota")
                    .with_stacktrace("em")
                    .with_unknown("desconhecido");

                let config = tenda_reporting::Config::default()
                    .with_index_type(tenda_reporting::IndexType::Byte)
                    .with_prefixes(prefixes);

                let mut rep_config = self.build_report_config();

                for hook in <Self as HasDiagnosticHooks<tenda_common::span::SourceSpan>>::hooks() {
                    rep_config = hook(self, rep_config);
                }

                let main_span = match &rep_config.span {
                    Some(sp) => sp.clone(),
                    None => {
                        panic!("No span found for report. Please ensure at least one #[span] is present.");
                    }
                };

                let mut main_label = tenda_reporting::Label::new(main_span.clone())
                    .with_color(tenda_reporting::Color::Red);

                if let Some(lbl) = main_span.label() {
                    main_label = main_label.with_message(lbl.clone());
                } else {
                    main_label = main_label.with_message(
                        format!("{}", "aqui".fg(tenda_reporting::Color::Red))
                    );
                }

                let mut builder = tenda_reporting::Report::build(kind, main_span.clone())
                    .with_config(config)
                    .with_message(rep_config.message)
                    .with_label(main_label);

                for lbl_span in rep_config.labels {
                    let mut label = tenda_reporting::Label::new(lbl_span.clone())
                        .with_color(tenda_reporting::Color::Red);

                    if let Some(lbl_txt) = lbl_span.label() {
                        label = label.with_message(lbl_txt.clone());
                    } else {
                        label = label.with_message(format!("{}", "aqui".fg(tenda_reporting::Color::Red)));
                    }
                    builder = builder.with_label(label);
                }

                for h in rep_config.helps {
                    builder = builder.with_help(h);
                }

                for n in rep_config.notes {
                    builder = builder.with_note(n);
                }


                builder
                    .with_stacktrace(rep_config.stacktrace)
                    .finish()
            }
        }
    };

    let maybe_hooks_impl = if accept_hooks {
        quote! {
            #[allow(dead_code)]
            const __REQUIRE_HOOKS_IMPL: () = {
                let _ = <#enum_ident as tenda_reporting::HasDiagnosticHooks<tenda_common::span::SourceSpan>>::hooks;
            };
        }
    } else {
        quote! {
            impl tenda_reporting::HasDiagnosticHooks<tenda_common::span::SourceSpan> for #enum_ident {
                fn hooks() -> &'static [fn(&Self, tenda_reporting::DiagnosticConfig<tenda_common::span::SourceSpan>) -> tenda_reporting::DiagnosticConfig<tenda_common::span::SourceSpan>] {
                    &[]
                }
            }
        }
    };

    let mut metadata_impls = Vec::new();

    for (field_ident, info) in metadata_map {
        let field_name_str = field_ident.to_string();
        let field_ty = match &info.field_type {
            Some(ty) => ty,
            None => continue,
        };

        let get_fn_name = Ident::new(&format!("get_{}", field_name_str), field_ident.span());
        let get_mut_fn_name =
            Ident::new(&format!("get_mut_{}", field_name_str), field_ident.span());
        let set_fn_name = Ident::new(&format!("set_{}", field_name_str), field_ident.span());

        let get_arms = &info.get_arms;
        let get_mut_arms = &info.get_mut_arms;
        let set_arms = &info.set_arms;

        let get_fn = quote! {
            pub fn #get_fn_name(&self) -> ::std::option::Option<&#field_ty> {
                match self {
                    #( #get_arms )*
                    _ => None
                }
            }
        };

        let get_mut_fn = quote! {
            pub fn #get_mut_fn_name(&mut self) -> ::std::option::Option<&mut #field_ty> {
                match self {
                    #( #get_mut_arms )*
                    _ => None
                }
            }
        };

        let set_fn = quote! {
            pub fn #set_fn_name(&mut self, value: #field_ty) {
                match self {
                    #( #set_arms )*
                    _ => {}
                }
            }
        };

        metadata_impls.push(get_fn);
        metadata_impls.push(get_mut_fn);
        metadata_impls.push(set_fn);
    }

    let metadata_impl = quote! {
        impl #enum_ident {
            #( #metadata_impls )*
        }
    };

    let expanded = quote! {
        #report_impl
        #maybe_hooks_impl
        #metadata_impl
    };

    expanded.into()
}
