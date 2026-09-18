//! Terminal-safe text. Anything a Source hands the screen (titles, prompts,
//! transcript lines) may carry escape sequences or bidi overrides; strip them
//! before they reach a cell.

/// Drop C0/C1 controls (keeping `\n` and `\t`), whole ESC sequences, and the
/// Unicode bidi overrides and isolates (U+202A..202E, U+2066..2069).
pub fn terminal_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\x1b' => skip_escape(&mut chars),
            '\n' | '\t' => out.push(c),
            '\u{0}'..='\u{1f}' | '\u{7f}'..='\u{9f}' => {}
            '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' => {}
            _ => out.push(c),
        }
    }
    out
}

/// Consume the rest of an escape sequence whose ESC was just read.
fn skip_escape(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    match chars.next() {
        // CSI: parameter and intermediate bytes, then one final byte 0x40..=0x7e.
        Some('[') => {
            for c in chars.by_ref() {
                if ('\u{40}'..='\u{7e}').contains(&c) {
                    break;
                }
            }
        }
        // OSC / DCS / SOS / PM / APC: runs to BEL or ST (ESC \).
        Some(']' | 'P' | 'X' | '^' | '_') => loop {
            match chars.next() {
                None | Some('\u{7}') => break,
                Some('\x1b') => {
                    if chars.peek() == Some(&'\\') {
                        chars.next();
                    }
                    break;
                }
                Some(_) => {}
            }
        },
        // Charset designations take one more byte; other two-byte sequences are done.
        Some('(' | ')' | '*' | '+' | '#' | '%') => {
            chars.next();
        }
        _ => {}
    }
}
