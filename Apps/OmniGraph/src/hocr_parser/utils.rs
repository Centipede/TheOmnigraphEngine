pub(crate) fn page_level() -> String {
    "page".to_string()
}

pub(crate) fn carea_level() -> String {
    "carea".to_string()
}

pub(crate) fn block_level() -> String {
    "block".to_string()
}

pub(crate) fn line_level() -> String {
    "line".to_string()
}

pub(crate) fn word_level() -> String {
    "word".to_string()
}

pub fn stem_from_id(id: &str) -> String {
    let stem = id.chars()
        .rev()
        .skip_while(|c| c.is_numeric() )
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    if stem.ends_with('_') {
        stem.chars()
            .rev()
            .skip_while(|c| *c == '_' )
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
    } else {
        stem
    }
}

pub fn count_from_id(id: &str) -> Result<usize, std::num::ParseIntError> {
    let counter = id
        .chars()
        .rev()
        .take_while(|c| c.is_numeric())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    counter.parse::<usize>()
}

pub(crate) fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn escape_attr(s: &str) -> String {
    escape_text(s).replace('"', "&quot;")
}
