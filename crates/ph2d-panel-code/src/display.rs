//! ⭐ **A linha como se MOSTRA** — tabs expandidas até à paragem, a pré-edição do IME inserida no
//! cursor e as dicas de tipo (I9) onde o assistente as pôs. A pintura molda ESTE texto; o clique e o cursor voltam ao buffer pelos mesmos mapas, por
//! isso o byte que a pintura pôs num sítio é o byte que o clique nesse sítio devolve.

use unicode_segmentation::UnicodeSegmentation;

/// O que uma expansão é.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExpKind {
    Tab,
    Preedit,
    /// Uma dica de tipo (I9): texto sem coluna nenhuma no buffer.
    Inlay,
}

/// Um trecho do buffer que ocupa outro tamanho no ecrã: uma tab (1 byte → `n` espaços), a
/// pré-edição ou uma dica (0 bytes → o texto delas).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Expansion {
    pub kind: ExpKind,
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
    /// `line` sem o `\n`; `preedit` = (byte relativo à linha, texto) do cursor primário; `inlays` =
    /// (byte relativo à linha, rótulo) das dicas, por ordem. Na mesma posição a pré-edição vem antes
    /// da dica (o que se escreve empurra-a, como no VS Code).
    pub fn build(
        line: &str,
        tab: usize,
        preedit: Option<(usize, &str)>,
        inlays: &[(usize, &str)],
    ) -> Self {
        let tab = tab.max(1);
        let mut out = Self {
            text: String::with_capacity(line.len()),
            exps: Vec::new(),
        };
        let mut w = 0usize;
        let mut pre = preedit.filter(|(_, s)| !s.is_empty());
        let mut hints = inlays.iter().filter(|(_, s)| !s.is_empty()).peekable();
        for (i, g) in line.grapheme_indices(true) {
            if let Some((p, s)) = pre
                && p <= i
            {
                w += out.push_preedit(i, s);
                pre = None;
            }
            while let Some((_, s)) = hints.next_if(|(at, _)| *at <= i) {
                out.push_zero(ExpKind::Inlay, i, s);
            }
            if g == "\t" {
                let n = tab - w % tab;
                out.exps.push(Expansion {
                    kind: ExpKind::Tab,
                    buf: i,
                    buf_len: 1,
                    disp: out.text.len(),
                    disp_len: n,
                });
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
        for (_, s) in hints {
            out.push_zero(ExpKind::Inlay, line.len(), s);
        }
        out
    }

    fn push_preedit(&mut self, at: usize, s: &str) -> usize {
        self.push_zero(ExpKind::Preedit, at, s);
        s.graphemes(true).count()
    }

    fn push_zero(&mut self, kind: ExpKind, at: usize, s: &str) {
        self.exps.push(Expansion {
            kind,
            buf: at,
            buf_len: 0,
            disp: self.text.len(),
            disp_len: s.len(),
        });
        self.text.push_str(s);
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

    /// Como [`Self::to_disp`], mas DEPOIS do que não tem bytes nessa posição (uma dica): onde uma faixa
    /// que começa em `b` começa a pintar-se.
    pub fn to_disp_after(&self, b: usize) -> usize {
        let mut d = b;
        for e in &self.exps {
            if e.buf > b || (e.buf == b && e.buf_len > 0) {
                break;
            }
            d = d + e.disp_len - e.buf_len;
        }
        d
    }

    /// O byte mostrado `d` é de uma dica (I9).
    pub fn is_inlay(&self, d: usize) -> bool {
        self.exps
            .iter()
            .any(|e| e.kind == ExpKind::Inlay && e.disp <= d && d < e.disp + e.disp_len)
    }

    /// As dicas: (byte mostrado do início, comprimento).
    pub fn inlays(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.exps
            .iter()
            .filter(|e| e.kind == ExpKind::Inlay)
            .map(|e| (e.disp, e.disp_len))
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
        self.exps
            .iter()
            .find(|e| e.kind == ExpKind::Preedit)
            .map(|e| (e.disp, e.disp_len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_ate_a_paragem_e_volta() {
        let d = DisplayLine::build("a\tb", 4, None, &[]);
        assert_eq!(d.text, "a   b");
        assert_eq!(d.to_disp(1), 1);
        assert_eq!(d.to_disp(2), 4);
        assert_eq!((d.owner(1), d.owner(3), d.owner(4)), (1, 1, 2));
    }

    #[test]
    fn preedit_no_meio() {
        let d = DisplayLine::build("ab", 4, Some((1, "é")), &[]);
        assert_eq!(d.text, "aéb");
        assert_eq!(d.to_disp(1), 1);
        assert_eq!(d.to_disp(2), 2 + "é".len());
        assert_eq!(d.owner(1), 1);
        assert_eq!(d.preedit_span(), Some((1, "é".len())));
    }

    #[test]
    fn a_dica_nao_tem_bytes_e_vem_depois_da_pre_edicao() {
        let d = DisplayLine::build("local s = 1", 4, Some((7, "x")), &[(7, ": number")]);
        assert_eq!(d.text, "local sx: number = 1");
        assert_eq!(d.to_disp(7), 7, "o cursor pinta-se ANTES da dica");
        assert_eq!(d.to_disp_after(7), 7 + 1 + ": number".len());
        assert_eq!(d.to_disp(8), d.to_disp_after(7) + 1);
        assert_eq!((d.owner(9), d.owner(d.to_disp(8))), (7, 8));
        assert!(d.is_inlay(8) && d.is_inlay(15) && !d.is_inlay(7) && !d.is_inlay(16));
        assert_eq!(d.inlays().collect::<Vec<_>>(), [(8, ": number".len())]);
        assert_eq!(d.preedit_span(), Some((7, 1)));
        let fim = DisplayLine::build("f()", 4, None, &[(3, ": ()")]);
        assert_eq!(fim.text, "f(): ()");
    }
}
