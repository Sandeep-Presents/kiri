//! The window's page as every system loads it: index.html with its icons
//! and the terminal emulator (xterm.js, vendored unmodified in `xterm/`)
//! spliced into their placeholders. The Mac and Linux window (crates/kiri)
//! and Windows (kiri-win includes this file) load the same string, and
//! scripts/ui-test.py builds it the same way.
//!
//! Its words: the English keys and the one language it opens in (the
//! `lang` setting's, or the system's for System; `lang::strings_js`), and the
//! system's languages and the saved pick (`lang::boot_js`). Another language
//! comes when the page switches to it: the page sends `langwords` and the
//! host evaluates `lang::words_js`.
//!
//! The page is loaded as a string, and WebView2's NavigateToString takes it
//! as a base64 data: URL of at most 2 MiB: 1.5 MiB of UTF-8
//! ([`fits_webview2`]). With xterm.js and the keys it is about 1.49 MB, and a
//! language adds its words: 60 to 150 KB, the most in scripts that take 2 or
//! 3 bytes a letter (Cyrillic, Greek, Arabic, Indic). All of them would be
//! well past the limit, which is why the page carries only one; and when that
//! one still takes it past, kiri-win opens the page with the English words
//! alone ([`page_english`]), and the page asks for its own as it starts.

const ICONS_JS: &str = include_str!("icons.js");
/// The languages and the first start's language step, which both pages show.
const LANGSTEP_JS: &str = include_str!("langstep.js");
const XTERM_CSS: &str = include_str!("xterm/xterm.css");
/// xterm.js 6.0.0 and its fit, search and web-links add-ons (UMD builds,
/// each defining a global), in the order they load.
const XTERM_JS: [&str; 4] = [include_str!("xterm/xterm.js"), include_str!("xterm/addon-fit.js"), include_str!("xterm/addon-search.js"), include_str!("xterm/addon-web-links.js")];

/// `index` (index.html) with everything spliced in, `lang` (the `lang`
/// setting) picking its language.
pub fn page(index: &str, lang: Option<&str>) -> String {
    assemble(index, lang, super::lang::resolve(lang))
}

/// [`page`] with the English words alone, `lang` still the one picked: the
/// page asks the host for that language's words (`langwords`) as it starts
/// and shows them a moment later, as when the language is switched.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn page_english(index: &str, lang: Option<&str>) -> String {
    assemble(index, lang, "en")
}

/// Whether WebView2's NavigateToString loads `page`: it takes it as a base64
/// data: URL of at most 2 MiB, and past that it fails and the window stays
/// blank (with WebView2 157, a page of 1,486,611 bytes loads and one of
/// 1,584,785 doesn't).
#[cfg_attr(not(windows), allow(dead_code))]
pub fn fits_webview2(page: &str) -> bool {
    page.len().div_ceil(3) * 4 <= 2 * 1024 * 1024
}

/// `index` with everything spliced in: `lang` picked, carrying the words of
/// `words` (a tag `lang::resolve` gave, or "en" for the keys alone).
fn assemble(index: &str, lang: Option<&str>, words: &str) -> String {
    let mut js = String::with_capacity(XTERM_JS.iter().map(|s| s.len() + 1).sum());
    for part in XTERM_JS {
        js.push_str(part);
        // Each ends in a source-map comment: the next must start on a line of its own.
        js.push('\n');
    }
    index
        .replacen("/*ICONS*/", ICONS_JS, 1)
        .replacen("/*LANGSTEP*/", LANGSTEP_JS, 1)
        .replacen("/*XTERM_CSS*/", XTERM_CSS, 1)
        .replacen("/*XTERM_JS*/", &js, 1)
        .replacen("/*LANG*/", &super::lang::boot_js(lang), 1)
        .replacen("/*STRINGS*/{}", &super::lang::strings_js(words), 1)
}

/// A page that only needs its words and the language step spliced in
/// (kiri-win's setup page).
#[cfg_attr(not(windows), allow(dead_code))]
pub fn words(page: &str, lang: Option<&str>) -> String {
    page.replacen("/*LANGSTEP*/", LANGSTEP_JS, 1).replacen("/*LANG*/", &super::lang::boot_js(lang), 1).replacen("/*STRINGS*/{}", &super::lang::strings_js(super::lang::resolve(lang)), 1)
}

#[cfg(test)]
mod tests {
    use super::super::lang;

    #[test]
    fn every_placeholder_is_filled_once_and_the_page_fits_webview2() {
        let index = include_str!("index.html");
        let page = super::page(index, None);
        for p in ["/*ICONS*/", "/*LANGSTEP*/", "/*XTERM_CSS*/", "/*XTERM_JS*/", "/*LANG*/", "/*STRINGS*/"] {
            assert!(!page.contains(p), "{p} left in the page");
        }
        assert!(page.contains(".xterm-viewport"), "xterm.css missing");
        assert!(page.contains("function langStep(") && page.contains("const LANGS = ["), "langstep.js missing");
        let setup = super::words(include_str!("../../../kiri-win/src/setup.html"), None);
        for p in ["/*LANGSTEP*/", "/*LANG*/", "/*STRINGS*/"] {
            assert!(!setup.contains(p), "{p} left in the setup page");
        }
        assert!(setup.contains("function langStep("), "langstep.js missing from the setup page");
        assert!(page.contains("window.KIRI_LANG = {") && page.contains(&lang::strings_js(lang::resolve(None))), "words missing");
        // Each language the page can open in carries the keys and that language
        // alone, and opens on Windows: in its own words when the page fits
        // NavigateToString, and in English, still picked, when it doesn't.
        for (tag, _) in lang::FILES {
            let p = super::page(index, Some(tag));
            assert!(p.contains(&lang::strings_js(tag)), "{tag}'s words missing");
            for (other, _) in lang::FILES {
                assert!(other == tag || !p.contains(&format!("\"{other}\":[")), "{tag}'s page carries {other} too");
            }
            if !super::fits_webview2(&p) {
                let english = super::page_english(index, Some(tag));
                assert!(super::fits_webview2(&english), "{tag}'s page in English is {} bytes", english.len());
                assert!(english.contains(&lang::boot_js(Some(tag))) && english.contains(&lang::strings_js("en")), "{tag} not picked in its English page");
            }
        }
        // English, which any page falls back to, keeps room to grow: its data:
        // URL stays 64 KiB under the limit.
        let english = super::page_english(index, None);
        assert!(english.len().div_ceil(3) * 4 + 64 * 1024 <= 2 * 1024 * 1024, "the English page is {} bytes", english.len());
        // Setup's page too.
        assert!(lang::FILES.iter().all(|(tag, _)| super::fits_webview2(&super::words(include_str!("../../../kiri-win/src/setup.html"), Some(tag)))));
    }
}
