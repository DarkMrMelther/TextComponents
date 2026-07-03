use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde_json::Value;
use std::fs;

use crate::translation::TranslationToken;

/// Count the number of parameters in a translation string
fn count_parameters(text: &str) -> usize {
    let sequential = text.matches("%s").count();
    let mut positional = 0;
    for i in 1..=8 {
        if text.contains(&format!("%{i}$s")) {
            positional = positional.max(i);
        }
    }
    sequential.max(positional)
}

fn process_tokens(text: &str) -> (TokenStream, i32) {
    let mut positions = vec![(0, 0, 0), (text.len(), 0, 0)];
    for i in 1..=8 {
        for (pos, _) in text.match_indices(&format!("%{i}$s")) {
            positions.push((pos, i, 4usize));
        }
    }
    for (counter, (pos, _)) in (1..).zip(text.match_indices("%s")) {
        positions.push((pos, counter, 2usize));
    }
    positions.sort_by_key(|(pos, _, _)| *pos);
    let mut positions = positions.into_iter().peekable();
    let mut stream = TokenStream::new();
    let mut amount = 0;
    while let Some((pos, _, size)) = positions.next() {
        let Some(next) = positions.peek() else {
            break;
        };
        let text = text[pos + size..next.0].to_string();
        let arg = next.1;
        stream.extend(quote! {
            TranslationToken::Text(Cow::borrow(#text)),
        });
        amount += 1;
        if arg > 0 {
            stream.extend(quote! {
                TranslationToken::Arg(#arg),
            });
            amount += 1;
        }
    }
    (quote! {[#stream]}, amount)
}

pub fn build_translations(path: &str) -> TokenStream {
    println!("cargo:rerun-if-changed={path}");

    let lang_file =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path} language file"));

    let translations: serde_json::Map<String, Value> =
        serde_json::from_str(&lang_file).unwrap_or_else(|_| panic!("Failed to parse {path}"));

    let mut stream = TokenStream::new();

    // Add imports
    stream.extend(quote! {
        #![allow(dead_code)]
        use text_components::{
            translation::{Translation, TranslationToken},
            build::TranslationsRegistry
        };
        use std::borrow::Cow;
    });

    // Generate constants for each translation
    let mut translations_vec: Vec<_> = translations.iter().collect();
    translations_vec.sort_by_key(|(k, _)| *k);

    // Track used constant names to handle collisions
    let mut used_names = rustc_hash::FxHashMap::default();
    let mut register_stream = TokenStream::new();

    for (key, value) in translations_vec {
        let Some(text) = value.as_str() else {
            eprintln!("Warning: Translation key '{key}' has non-string value, skipping");
            continue;
        };

        let param_count = count_parameters(text);

        // Skip translations with more than 8 parameters
        if param_count > 8 {
            eprintln!(
                "Warning: Translation '{key}' has {param_count} parameters (max 8 supported), skipping"
            );
            continue;
        }

        let mut const_name_str = key.to_shouty_snake_case();

        // Handle collisions by appending a number
        if let Some(count) = used_names.get_mut(&const_name_str) {
            *count += 1;
            const_name_str = format!("{const_name_str}_{count}");
        } else {
            used_names.insert(const_name_str.clone(), 0);
        }

        let const_name = Ident::new(&const_name_str, Span::call_site());
        let (translation_tokens, tokens_amount) = process_tokens(text);
        let token_const_name = Ident::new(&format!("{const_name_str}_TOKEN"), Span::call_site());

        stream.extend(quote! {
            #[doc = #text]
            pub static #const_name: Translation<#param_count> = Translation(#key);
            static #token_const_name: [TranslationToken<'static>; #tokens_amount] = #translation_tokens;
        });

        register_stream.extend(quote! {
            registry.register("en", #key, &#token_const_name);
        });
    }

    stream.extend(quote! {
        fn register_translations<R: TranslationsRegistry>(registry: &mut R) {
            #register_stream
        }
    });

    stream
}

pub trait TranslationsRegistry {
    fn register(
        &mut self,
        locale: &'static str,
        key: &'static str,
        translation: &'static [TranslationToken<'static>],
    );
}
