use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr, Type};

/// Derive macro for creating a `Report` implementation that supports:
///
/// - `#[span]` for the primary span
/// - `#[label]` for extra spans (each field can be `Option<SourceSpan>`, `SourceSpan`, or `Vec<SourceSpan>`)
/// - `#[help]` and `#[note]` for textual messages (each field can be `Option<String>` or `Vec<String>`)
/// - `#[message]` for overriding the default error message (`String` or `Option<String>`)
/// - A top-level enum attribute `#[report("error_kind")]` to customize the error kind
#[proc_macro_derive(Report, attributes(report, span, label, help, note, message))]
pub fn report_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

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

    for variant in &enum_data.variants {
        let variant_ident = &variant.ident;

        let mut get_span_arm = quote! { #enum_ident::#variant_ident { .. } => None };
        let mut set_span_arm = quote! { #enum_ident::#variant_ident { .. } => {} };
        let mut get_label_arm = quote! { #enum_ident::#variant_ident { .. } => Vec::new() };
        let mut get_help_arm = quote! { #enum_ident::#variant_ident { .. } => Vec::new() };
        let mut get_note_arm = quote! { #enum_ident::#variant_ident { .. } => Vec::new() };
        let mut get_message_arm = quote! { #enum_ident::#variant_ident { .. } => None };

        if let Fields::Named(named_fields) = &variant.fields {
            let mut span_fields = Vec::new();
            let mut label_fields = Vec::new();
            let mut help_fields = Vec::new();
            let mut note_fields = Vec::new();
            let mut message_fields = Vec::new();

            for field in &named_fields.named {
                let ident = field.ident.as_ref().unwrap();
                let mut is_span_field = false;
                let mut is_label_field = false;
                let mut is_help_field = false;
                let mut is_note_field = false;
                let mut is_message_field = false;

                for attr in &field.attrs {
                    if attr.path().is_ident("span") {
                        is_span_field = true;
                    } else if attr.path().is_ident("label") {
                        is_label_field = true;
                    } else if attr.path().is_ident("help") {
                        is_help_field = true;
                    } else if attr.path().is_ident("note") {
                        is_note_field = true;
                    } else if attr.path().is_ident("message") {
                        is_message_field = true;
                    }
                }

                if is_span_field {
                    if let Type::Path(type_path) = &field.ty {
                        if let Some(last_seg) = type_path.path.segments.last() {
                            match last_seg.ident.to_string().as_str() {
                                "Option" => {
                                    span_fields.push((
                                        ident.clone(),
                                        quote! { #enum_ident::#variant_ident { #ident, .. } => #ident.clone() },
                                        quote! {
                                            #enum_ident::#variant_ident { #ident, .. } => {
                                                *#ident = Some(new_span.clone());
                                            }
                                        }
                                    ));
                                }
                                "SourceSpan" => {
                                    span_fields.push((
                                        ident.clone(),
                                        quote! { #enum_ident::#variant_ident { #ident, .. } => Some(#ident.clone()) },
                                        quote! {
                                            #enum_ident::#variant_ident { #ident, .. } => {
                                                *#ident = new_span.clone();
                                            }
                                        }
                                    ));
                                }
                                _ => {
                                    return syn::Error::new_spanned(
                                        &field.ty,
                                        "Expected `#[span]` field to be `SourceSpan` or `Option<SourceSpan>`",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                            }
                        }
                    }
                }

                if is_label_field {
                    if let Type::Path(type_path) = &field.ty {
                        if let Some(last_seg) = type_path.path.segments.last() {
                            match last_seg.ident.to_string().as_str() {
                                "Option" => {
                                    label_fields.push((ident.clone(), "OptionSpan".to_string()));
                                }
                                "Vec" => {
                                    label_fields.push((ident.clone(), "VecSpan".to_string()));
                                }
                                "SourceSpan" => {
                                    label_fields.push((ident.clone(), "DirectSpan".to_string()));
                                }
                                _ => {
                                    return syn::Error::new_spanned(
                                        &field.ty,
                                        "Expected `#[label]` field to be `SourceSpan`, `Option<SourceSpan>`, or `Vec<SourceSpan>`",
                                    )
                                    .to_compile_error()
                                    .into();
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

                if is_help_field {
                    match check_help_note(&field.ty) {
                        Ok(kind) => help_fields.push((ident.clone(), kind)),
                        Err(err) => return err.to_compile_error().into(),
                    }
                }

                if is_note_field {
                    match check_help_note(&field.ty) {
                        Ok(kind) => note_fields.push((ident.clone(), kind)),
                        Err(err) => return err.to_compile_error().into(),
                    }
                }

                if is_message_field {
                    if let Type::Path(type_path) = &field.ty {
                        if let Some(last_seg) = type_path.path.segments.last() {
                            match last_seg.ident.to_string().as_str() {
                                "String" => {
                                    message_fields
                                        .push((ident.clone(), "DirectString".to_string()));
                                }
                                "Option" => {
                                    message_fields
                                        .push((ident.clone(), "OptionString".to_string()));
                                }
                                _ => {
                                    return syn::Error::new_spanned(
                                        &field.ty,
                                        "Expected `#[message]` field to be `String` or `Option<String>`",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                            }
                        }
                    }
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
                    #enum_ident::#variant_ident { #(ref #all_idents),* } => {
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
                    #enum_ident::#variant_ident { #(ref #all_idents),* } => {
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
                            // "VecString"
                            quote! {
                                notes.extend(#ident.clone());
                            }
                        }
                    })
                    .collect();

                let note_arm = quote! {
                    #enum_ident::#variant_ident { #(ref #all_idents),* } => {
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
                        #enum_ident::#variant_ident { #ident, .. } => {
                            Some(#ident.clone())
                        }
                    },
                    "OptionString" => quote! {
                        #enum_ident::#variant_ident { #ident, .. } => {
                            #ident.clone()
                        }
                    },
                    _ => quote! {
                        #enum_ident::#variant_ident { .. } => None
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

    let expanded = quote! {
        impl common::report::Report for #enum_ident {
            fn get_span(&self) -> ::std::option::Option<common::span::SourceSpan> {
                match self {
                    #( #get_span_arms ),*
                }
            }

            fn set_span(&mut self, new_span: &common::span::SourceSpan) {
                match self {
                    #( #set_span_arms ),*
                }
            }

            fn get_labels(&self) -> ::std::vec::Vec<common::span::SourceSpan> {
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

            fn to_report(&self) -> ariadne::Report<common::span::SourceSpan> {
                use ariadne::Fmt;

                let fallback_message = self.to_string();

                let main_span = self
                    .get_span()
                    .expect("`to_report()` called on error variant without a main #[span]");

                let label_spans = self.get_labels();
                let helps = self.get_helps();
                let notes = self.get_notes();

                let final_message = if let Some(custom) = self.get_message() {
                    custom
                } else {
                    fallback_message
                };

                let kind = ariadne::ReportKind::Custom(&#error_kind, ariadne::Color::Red);

                let prefixes = ariadne::Prefixes::new()
                    .with_help("ajuda")
                    .with_note("nota");

                let config = ariadne::Config::default()
                    .with_index_type(ariadne::IndexType::Byte)
                    .with_prefixes(prefixes);

                let mut builder = ariadne::Report::build(kind, main_span.clone())
                    .with_config(config)
                    .with_message(final_message)
                    .with_label(
                        ariadne::Label::new(main_span.clone())
                            .with_message(format!("{}", "aqui".fg(ariadne::Color::Red)))
                            .with_color(ariadne::Color::Red)
                    );

                for lbl_span in label_spans {
                    let mut label =
                        ariadne::Label::new(lbl_span.clone())
                            .with_color(ariadne::Color::Yellow);

                    if let Some(lbl) = lbl_span.label() {
                        label = label.with_message(lbl.clone());
                    } else {
                        label = label.with_message(format!("{}", "aqui".fg(ariadne::Color::Yellow)));
                    }

                    builder = builder.with_label(label);
                }

                for help_msg in helps {
                    builder = builder.with_help(help_msg);
                }

                for note_msg in notes {
                    builder = builder.with_note(note_msg);
                }

                builder.finish()
            }
        }
    };

    expanded.into()
}
