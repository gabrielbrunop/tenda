use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(Report)]
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

    let enum_ident = &input.ident;

    let mut get_span_arms = Vec::new();
    let mut set_span_arms = Vec::new();
    let mut get_help_arms = Vec::new();

    for variant in &enum_data.variants {
        let variant_ident = &variant.ident;

        let mut get_span_arm = quote! { #enum_ident::#variant_ident { .. } => None };
        let mut set_span_arm = quote! { #enum_ident::#variant_ident { .. } => {} };
        let mut get_help_arm = quote! { #enum_ident::#variant_ident { .. } => None };

        if let Fields::Named(named_fields) = &variant.fields {
            for field in &named_fields.named {
                if let Some(field_ident) = &field.ident {
                    if field_ident == "span" {
                        if let Type::Path(type_path) = &field.ty {
                            if let Some(last_segment) = type_path.path.segments.last() {
                                if last_segment.ident == "Option" {
                                    get_span_arm = quote! {
                                        #enum_ident::#variant_ident { span, .. } => span.clone()
                                    };
                                    set_span_arm = quote! {
                                        #enum_ident::#variant_ident { span, .. } => {
                                            *span = Some(new_span.clone());
                                        }
                                    };
                                } else if last_segment.ident == "SourceSpan" {
                                    get_span_arm = quote! {
                                        #enum_ident::#variant_ident { span, .. } => Some(span.clone())
                                    };
                                    set_span_arm = quote! {
                                        #enum_ident::#variant_ident { span, .. } => {
                                            *span = new_span.clone();
                                        }
                                    };
                                } else {
                                    return syn::Error::new_spanned(
                                        &field.ty,
                                        "Expected `span: SourceSpan` or `span: Option<SourceSpan>`",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                            }
                        }
                    }

                    if field_ident == "help" {
                        if let Type::Path(type_path) = &field.ty {
                            if let Some(last_segment) = type_path.path.segments.last() {
                                if last_segment.ident == "Option" {
                                    get_help_arm = quote! {
                                        #enum_ident::#variant_ident { help, .. } => help.clone()
                                    };
                                } else {
                                    return syn::Error::new_spanned(
                                        &field.ty,
                                        "Expected `help: Option<String>`",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                            }
                        }
                    }
                }
            }
        }

        get_span_arms.push(get_span_arm);
        set_span_arms.push(set_span_arm);
        get_help_arms.push(get_help_arm);
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

            fn get_help(&self) -> ::std::option::Option<String> {
                match self {
                    #( #get_help_arms ),*
                }
            }

            fn to_report(&self) -> ariadne::Report<common::span::SourceSpan> {
                use ariadne::Fmt;

                let message = self.to_string();
                let span = self
                    .get_span()
                    .expect("`to_report()` called on error variant without a span");
                let help = self.get_help();

                let kind = ariadne::ReportKind::Custom("erro", ariadne::Color::Red);

                let mut builder = ariadne::Report::build(kind, span.clone())
                    .with_message(message)
                    .with_label(
                        ariadne::Label::new(span.clone())
                            .with_message(format!("{}", "aqui".fg(ariadne::Color::Red)))
                            .with_color(ariadne::Color::Red)
                    );

                if let Some(help) = help {
                    builder = builder.with_help(help);
                }

                builder.finish()
            }
        }
    };

    expanded.into()
}
