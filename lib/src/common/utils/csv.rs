pub fn detect_separator(text: &str) -> char {
    const DELIMITERS: &[char] = &['|', '\t', ','];

    text.chars()
        .find(|candidates| DELIMITERS.contains(candidates))
        .unwrap_or(',') // Default
}

pub fn scrub(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .replace(['\u{feff}', '\0'], "")       // Strip Null bytes (UTF-16 faux-ASCII)
        .replace("\r\n", "\n")   // Normalize Windows CRLF
        .replace('\r', "\n")            // Normalize Mac Classic
}