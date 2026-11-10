mod lang;

use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;

const EVDEV_LST: &str = "/usr/share/X11/xkb/rules/evdev.lst";

/// layouts left out of the list, `custom` is a placeholder for a user-written layout and empty
/// unless someone put one there
const SKIPPED_LAYOUTS: &[&str] = &["custom"];

/// the `(id, description)` lines under each `! section` header of an xkb `.lst` file
fn lst_sections(file: &str) -> std::collections::HashMap<&str, Vec<(&str, &str)>> {
    let mut sections = std::collections::HashMap::<_, Vec<_>>::new();
    let mut section = None;
    for line in file.lines() {
        if let Some(name) = line.strip_prefix('!') {
            section = Some(name.trim());
            continue;
        }
        let (Some(section), Some((id, desc))) =
            (section, line.trim().split_once(char::is_whitespace))
        else {
            continue;
        };
        sections.entry(section).or_default().push((id, desc.trim()));
    }
    sections
}

/// Generate the `KEYMAP` static map.
///
/// This proc_macro is modified from this message:
/// <https://discord.com/channels/273534239310479360/1120124565591425034/1324394178133884928>
///
/// Big thanks to (gh) @circles-png for helping!
#[proc_macro]
pub fn keymap(static_name: TokenStream) -> TokenStream {
    let static_name = quote::format_ident!("{}", static_name.to_string());
    let file = match std::fs::read_to_string(EVDEV_LST) {
        Ok(file) => file,
        Err(e) => {
            let msg = format!("cannot read {EVDEV_LST}: {e}");
            return quote! { compile_error!(#msg); }.into();
        }
    };
    let sections = lst_sections(&file);
    let (Some(layouts), Some(variants)) = (sections.get("layout"), sections.get("variant")) else {
        let msg = format!("{EVDEV_LST} has no `! layout` or `! variant` section");
        return quote! { compile_error!(#msg); }.into();
    };
    // a variant's description is `layout: name`
    let mut by_layout = std::collections::HashMap::<_, Vec<_>>::new();
    for &(variant_id, desc) in variants {
        let Some((layout_id, name)) = desc.split_once(": ") else {
            let msg = format!("{EVDEV_LST}: variant `{variant_id}` has no layout: {desc:?}");
            return quote! { compile_error!(#msg); }.into();
        };
        by_layout.entry(layout_id).or_default().push(quote! { #variant_id => #name, });
    }
    let layouts =
        layouts.iter().filter(|(id, _)| !SKIPPED_LAYOUTS.contains(id)).map(|(id, name)| {
            let mut variants = by_layout.remove(id).unwrap_or_default();
            variants.insert(0, quote! { "default" => "Default", });
            quote! {
                #id => Layout {
                    name: #name,
                    variants: phf::phf_ordered_map! {
                        #(#variants)*
                    },
                },
            }
        });
    let layouts = layouts.collect_vec();

    quote! {
        // so cargo rebuilds when xkeyboard-config changes the list
        const _: &[u8] = include_bytes!(#EVDEV_LST);

        #[derive(Debug)]
        pub struct Layout {
            pub name: &'static str,
            pub variants: phf::OrderedMap<&'static str, &'static str>,
        }
        pub static #static_name: phf::OrderedMap<&'static str, Layout> = phf::phf_ordered_map! {
            #(#layouts)*
        };
    }
    .into()
}

#[proc_macro]
pub fn comptime_localedef_langrows(const_name: TokenStream) -> TokenStream {
    let const_name = quote::format_ident!("{}", const_name.to_string());
    let mut langs = lang::LanguageRow::list();

    // sort the popular languages, put to top
    for lang in lang::POPULAR_LANGS.iter().rev() {
        let Some(index) = langs.iter().position(|l| l.locale == *lang) else {
            continue;
        };
        let x = langs.remove(index);
        langs.insert(0, x);
    }

    langs.push(lang::LanguageRow {
        locale: "en-owo".to_owned(),
        native_name: "OWO".to_string(),
        en_name: "OWO".to_string(),
    });

    let langs = langs.into_iter().map(|lang::LanguageRow { locale, native_name, en_name }| {
        quote! {
            LanguageRow {
                locale: #locale,
                native_name: #native_name,
                en_name: #en_name,
            },
        }
    });
    let len = langs.len();

    quote! {
        pub const #const_name: [LanguageRow; #len] = [#(#langs)*];
    }
    .into()
}
