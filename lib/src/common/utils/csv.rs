#[allow(dead_code)]
pub fn detect_separator(text: &str) -> char {
    if text.contains('|') { return '|'; }
    if text.contains('\t') { return '\t'; }
    ',' // Default
}

pub fn scrub(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .replace('\u{feff}', "") // Strip BOM
        .replace('\0', "")       // Strip Null bytes (UTF-16 faux-ASCII)
        .replace("\r\n", "\n")   // Normalize Windows CRLF
        .replace('\r', "\n")            // Normalize Mac Classic
}