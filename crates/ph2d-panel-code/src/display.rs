//! ⭐ **A linha como se MOSTRA** — tabs expandidas até à paragem e a pré-edição do IME inserida no
//! cursor. A pintura molda ESTE texto; o clique e o cursor voltam ao buffer pelos mesmos mapas, por
//! isso o byte que a pintura pôs num sítio é o byte que o clique nesse sítio devolve.

use unicode_segmentation::UnicodeSegmentation;

/// Um trecho do buffer que ocupa outro tamanho no ecrã: uma tab (1 byte → `n` espaços) ou a
/// pré-edição (0 bytes → o texto composto).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Expansion {
    /// Byte (relativo à linha) onde a expansão está.
    pub buf: usize,
    pub buf_len: usize,
    /// Byte (no texto mostrado) onde ela começa.
    pub disp: usize,
    pub disp_len: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DisplayLine {
    pub text: String,
    pub exps: Vec<Expansion>,
}

impl DisplayLine {
    /// `line` sem o `\n`; `preedit` = (byte relativo à linha, texto) do cursor primário.
    pub fn build(line: &str, tab: usize, preedit: Option<(usize, &str)>) -> Self {
        let tab = tab.max(1);
        let mut out = Self { text: String::with_capacity(line.len()), exps: Vec::new() };
        let mut w = 0usize;
        let mut pre = preedit.filter(|(_, s)| !s.is_empty());
        for (i, g) in line.grapheme_indices(true) {
            if let Some((p, s)) = pre
                && p <= i
            {
                w += out.push_preedit(i, s);
                pre = None;
            }
            if g == "\t" {
                let n = tab - w % tab;
                out.exps.push(Expansion { buf: i, buf_len: 1, disp: out.text.len(), disp_len: n });
                out.text.extend(std::iter::repeat_n(' ', n));
                w += n;
            } else {
                out.text.push_str(g);
                w += 1;
            }
        }
        if let Some((_, s)) = pre {
            out.push_preedit(line.len(), s);
        }
        out
    }

    fn push_preedit(&mut self, at: usize, s: &str) -> usize {
        self.exps.push(Expansion { buf: at, buf_len: 0, disp: self.text.len(), disp_len: s.len() });
        self.text.push_str(s);
        s.graphemes(true).count()
    }

    /// Byte do buffer (relativo à linha) → byte mostrado. Na posição da pré-edição dá o ANTES dela.
    pub fn to_disp(&self, b: usize) -> usize {
        let mut d = b;
        for e in &self.exps {
            if e.buf >= b {
                break;
            }
            d = d + e.disp_len - e.buf_len;
        }
        d
    }

    /// O byte do buffer DONO do byte mostrado `d` (os espaços de uma tab são da tab; a pré-edição é
    /// do byte onde está). `d == text.len()` dá o fim da linha.
    pub fn owner(&self, d: usize) -> usize {
        let mut delta = 0usize;
        for e in &self.exps {
            if d < e.disp {
                break;
            }
            if d < e.disp + e.disp_len {
                return e.buf;
            }
            delta += e.disp_len - e.buf_len;
        }
        d - delta
    }

    /// Onde a pré-edição está (byte mostrado do início e o comprimento), se há.
    pub fn preedit_span(&self) -> Option<(usize, usize)> {
        self.exps.iter().find(|e| e.buf_len == 0).map(|e| (e.disp, e.disp_len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_ate_a_paragem_e_volta() {
        let d = DisplayLine::build("a\tb", 4, None);
        assert_eq!(d.text, "a   b");
        assert_eq!(d.to_disp(1), 1);
        assert_eq!(d.to_disp(2), 4);
        assert_eq!((d.owner(1), d.owner(3), d.owner(4)), (1, 1, 2));
    }

    #[test]
    fn preedit_no_meio() {
        let d = DisplayLine::build("ab", 4, Some((1, "é")));
        assert_eq!(d.text, "aéb");
        assert_eq!(d.to_disp(1), 1);
        assert_eq!(d.to_disp(2), 2 + "é".len());
        assert_eq!(d.owner(1), 1);
        assert_eq!(d.preedit_span(), Some((1, "é".len())));
    }
}
