use icu::experimental::displaynames;
// This is a list of languages sorted by total speakers:
// https://en.wikipedia.org/wiki/List_of_languages_by_total_number_of_speakers
// (2024-08-17)
//
// These are filtered by our Ultramarine website plausible statistics and the 5 most popular
// langauges in the world.
#[expect(dead_code, reason = "the popular-first sorting in lib.rs is disabled for now")]
pub const POPULAR_LANGS: [&str; 9] =
    ["en_US", "zh_CN", "zh_TW", "hi_IN", "es_ES", "ar_AE", "fr_FR", "pt_BR", "de_DE"];
#[derive(Clone, Debug)]

pub struct LanguageRow {
    pub locale: String,
    pub native_name: String,
}
impl LanguageRow {
    pub fn list() -> Vec<Self> {
        let mut displaynameopt = displaynames::DisplayNamesOptions::default();
        displaynameopt.style = Some(displaynames::Style::Short);
        // FIXME: maybe use some C API instead?
        let mut cmd = std::process::Command::new("localedef");
        cmd.arg("--list-archive").stdout(std::process::Stdio::piped());
        let mut stdout = cmd.output().expect("cannot run localedef").stdout;
        (stdout.split_mut(|&b| b == b'\n'))
            .filter(|v| !v.contains(&b'.') && !v.contains(&b'@') && !v.is_empty())
            // legacy glibc aliases (`bokmal`, `deutsch`, …) aren't locales, real ones have a
            // territory (`en_US`) or are a bare 2–3 letter language code (`eo`)
            .filter(|v| v.contains(&b'_') || v.len() <= 3)
            .filter_map(|locale| {
                locale.iter_mut().filter(|c| **c == b'_').for_each(|c| *c = b'-');
                let locale = icu::locale::Locale::try_from_utf8(locale)
                    .inspect_err(|e| println!("cannot parse locale: {locale:?}: {e}"))
                    .ok()?;
                if locale == icu::locale::Locale::UNKNOWN {
                    return None;
                };
                let displaynamehdl = displaynames::multi::LocaleDisplayNamesFormatter::try_new(
                    (&locale).into(),
                    displaynameopt,
                )
                .ok()?;

                Some(Self {
                    locale: locale.to_string(),
                    native_name: displaynamehdl.of(&locale).to_string(),
                })
            })
            .collect()
    }
}
