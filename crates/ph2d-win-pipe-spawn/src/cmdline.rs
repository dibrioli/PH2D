//! A linha de comando do `CreateProcessW`, pelas regras do `CommandLineToArgvW` (as do CRT): o
//! programa entre aspas tal como é (um caminho do Windows nunca tem `"`), cada argumento com espaço,
//! tab ou aspas entre aspas, as `\` antes de `"` dobradas.

const QUOTE: u16 = b'"' as u16;
const BACKSLASH: u16 = b'\\' as u16;
const SPACE: u16 = b' ' as u16;
const TAB: u16 = b'\t' as u16;

/// `program` e `args` (UTF-16, sem NUL) → a linha de comando terminada em NUL.
#[must_use]
pub fn command_line(program: &[u16], args: &[Vec<u16>]) -> Vec<u16> {
    let mut out = Vec::new();
    out.push(QUOTE);
    out.extend_from_slice(program);
    out.push(QUOTE);
    for a in args {
        out.push(SPACE);
        push_arg(&mut out, a);
    }
    out.push(0);
    out
}

fn push_arg(out: &mut Vec<u16>, arg: &[u16]) {
    if !arg.is_empty() && !arg.iter().any(|&c| c == SPACE || c == TAB || c == QUOTE) {
        out.extend_from_slice(arg);
        return;
    }
    out.push(QUOTE);
    let mut backslashes = 0usize;
    for &c in arg {
        if c == BACKSLASH {
            backslashes += 1;
        } else {
            if c == QUOTE {
                out.extend(std::iter::repeat_n(BACKSLASH, backslashes + 1));
            }
            backslashes = 0;
        }
        out.push(c);
    }
    out.extend(std::iter::repeat_n(BACKSLASH, backslashes));
    out.push(QUOTE);
}
