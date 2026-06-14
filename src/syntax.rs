#![cfg_attr(not(feature = "ssr"), allow(unused_imports, dead_code))]
use std::sync::OnceLock;

#[cfg(feature = "ssr")]
pub fn syntax_set() -> &'static syntect::parsing::SyntaxSet {
    static SS: OnceLock<syntect::parsing::SyntaxSet> = OnceLock::new();
    SS.get_or_init(|| syntect::parsing::SyntaxSet::load_defaults_newlines())
}

#[cfg(feature = "ssr")]
pub fn theme_set() -> &'static syntect::highlighting::ThemeSet {
    static TS: OnceLock<syntect::highlighting::ThemeSet> = OnceLock::new();
    TS.get_or_init(|| syntect::highlighting::ThemeSet::load_defaults())
}

#[cfg(feature = "ssr")]
pub fn highlight(code: &str, extension: &str) -> String {
    let ss = syntax_set();
    let ts = theme_set();

    let syntax = ss
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme = ts
        .themes
        .get("base16-ocean.dark")
        .unwrap_or_else(|| ts.themes.values().next().unwrap());

    syntect::html::highlighted_html_for_string(code, ss, syntax, theme)
        .unwrap_or_else(|e| format!("<pre>Highlight error: {e}</pre>"))
}

#[cfg(feature = "ssr")]
pub fn highlight_line(code: &str, extension: &str) -> String {
    let ss = syntax_set();
    let ts = theme_set();

    let syntax = ss
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme = ts
        .themes
        .get("base16-ocean.dark")
        .unwrap_or_else(|| ts.themes.values().next().unwrap());

    let mut highlighter =
        syntect::easy::HighlightLines::new(syntax, theme);

    let ranges = highlighter
        .highlight_line(code, ss)
        .unwrap_or_default();

    syntect::html::styled_line_to_highlighted_html(
        &ranges,
        syntect::html::IncludeBackground::No,
    )
    .unwrap_or_else(|_| code.to_string())
}
