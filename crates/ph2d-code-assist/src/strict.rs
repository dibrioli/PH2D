//! `--!strict` por ficheiro (I14): onde está a diretiva do modo estrito do Luau.

use ph2d_code_buffer::Text;

/// A diretiva do modo estrito do Luau.
pub const STRICT: &str = "--!strict";

/// A linha da diretiva, se há: os comentários de topo (`--!` são as diretivas) antes do 1.º código.
#[must_use]
pub fn directive_line(text: &Text) -> Option<usize> {
    for line in 0..text.line_count() {
        let l = text.slice(text.byte_of_line(line)..text.line_end(line));
        let l = l.trim();
        if l == STRICT {
            return Some(line);
        }
        if !(l.is_empty() || l.starts_with("--")) {
            return None;
        }
    }
    None
}

/// O script tem `--!strict`.
#[must_use]
pub fn is_strict(text: &Text) -> bool {
    directive_line(text).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_diretiva_so_conta_antes_do_codigo() {
        let s = |t: &str| is_strict(&Text::new(t));
        assert!(s("--!strict\nlocal x = 1\n"));
        assert!(s("-- bob\n\n--!native\n--!strict\nlocal x = 1\n"));
        assert!(!s("local x = 1\n--!strict\n"));
        assert!(!s("--!nonstrict\nlocal x = 1\n"));
        assert!(!s("-- --!strict\n"));
    }
}
