use std::sync::OnceLock;

pub fn syntax_set() -> &'static syntect::parsing::SyntaxSet {
    static SS: OnceLock<syntect::parsing::SyntaxSet> = OnceLock::new();
    SS.get_or_init(|| syntect::parsing::SyntaxSet::load_defaults_newlines())
}

pub fn theme_set() -> &'static syntect::highlighting::ThemeSet {
    static TS: OnceLock<syntect::highlighting::ThemeSet> = OnceLock::new();
    TS.get_or_init(|| syntect::highlighting::ThemeSet::load_defaults())
}

const SYNTAX_COLORS: &[(&str, &str)] = &[
    ("#c0c5ce", "var(--sx-text)"),
    ("#a7adba", "var(--sx-text)"),
    ("#dfe1e8", "var(--sx-text)"),
    ("#eff1f5", "var(--sx-text)"),
    ("#65737e", "var(--sx-comment)"),
    ("#4f5b66", "var(--sx-comment)"),
    ("#bf616a", "var(--sx-keyword)"),
    ("#d08770", "var(--sx-number)"),
    ("#ebcb8b", "var(--sx-string)"),
    ("#a3be8c", "var(--sx-string)"),
    ("#96b5b4", "var(--sx-builtin)"),
    ("#8fa1b3", "var(--sx-function)"),
    ("#b48ead", "var(--sx-type)"),
    ("#ab7967", "var(--sx-constant)"),
];

fn replace_syntax_colors(html: &str) -> String {
    let mut result = html.to_string();
    for &(hex, var) in SYNTAX_COLORS {
        result = result.replace(
            &format!("style=\"color:{hex};\""),
            &format!("style=\"color:{var};\""),
        );
    }
    result
}

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

    let html = syntect::html::highlighted_html_for_string(code, ss, syntax, theme)
        .unwrap_or_else(|e| format!("<pre>Highlight error: {e}</pre>"));

    replace_syntax_colors(&html)
}

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

    let html = syntect::html::styled_line_to_highlighted_html(
        &ranges,
        syntect::html::IncludeBackground::No,
    )
    .unwrap_or_else(|_| code.to_string());

    replace_syntax_colors(&html)
}
