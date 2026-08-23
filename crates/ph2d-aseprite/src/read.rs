//! **O cursor little-endian** sobre os bytes do ficheiro.
//!
//! ⚠️ Todo acessor devolve `Option` e **nunca** entra em pânico: a entrada é um ficheiro que o
//! utilizador largou na janela, e um `.ase` truncado (um download interrompido, um disco cheio a
//! meio de um save) tem de virar uma mensagem, não um crash do app.

pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub(crate) fn pos(&self) -> usize {
        self.pos
    }

    pub(crate) fn seek(&mut self, to: usize) -> Option<()> {
        (to <= self.bytes.len()).then(|| self.pos = to)
    }

    pub(crate) fn skip(&mut self, n: usize) -> Option<()> {
        let to = self.pos.checked_add(n)?;
        self.seek(to)
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let s = self.bytes.get(self.pos..end)?;
        self.pos = end;
        Some(s)
    }

    pub(crate) fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|s| s[0])
    }

    pub(crate) fn u16(&mut self) -> Option<u16> {
        self.take(2).map(|s| u16::from_le_bytes([s[0], s[1]]))
    }

    pub(crate) fn i16(&mut self) -> Option<i16> {
        self.u16().map(|v| v as i16)
    }

    pub(crate) fn u32(&mut self) -> Option<u32> {
        self.take(4)
            .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }

    /// Uma STRING do formato: `WORD` de comprimento + esse número de bytes UTF-8.
    ///
    /// ⚠️ **`from_utf8_lossy`, e não uma recusa.** O nome de uma camada ou de uma tag vem do
    /// teclado de outra pessoa; recusar o ficheiro inteiro por um byte inválido num nome trocaria
    /// um desenho importado por nada.
    pub(crate) fn string(&mut self) -> Option<String> {
        let n = usize::from(self.u16()?);
        let s = self.take(n)?;
        Some(String::from_utf8_lossy(s).into_owned())
    }

    /// Um sub-leitor sobre os próximos `n` bytes, avançando este. É assim que o corpo de um chunk
    /// fica **fechado** — ler a mais dentro dele devolve `None` em vez de invadir o chunk seguinte.
    pub(crate) fn window(&mut self, n: usize) -> Option<Reader<'a>> {
        self.take(n).map(Reader::new)
    }

    /// O que sobra, sem copiar — o payload de um cel.
    pub(crate) fn rest(&mut self) -> &'a [u8] {
        let s = &self.bytes[self.pos..];
        self.pos = self.bytes.len();
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Ler para lá do fim devolve `None`, não entra em pânico** — a lei inteira deste ficheiro.
    #[test]
    fn every_read_past_the_end_is_none() {
        let mut r = Reader::new(&[1, 2, 3]);
        assert_eq!(r.u16(), Some(0x0201));
        assert_eq!(r.u16(), None, "so' sobrava um byte");
        assert_eq!(r.u8(), Some(3));
        assert_eq!(r.u8(), None);
        assert_eq!(r.u32(), None);
        assert!(r.skip(1).is_none());
        assert!(r.seek(99).is_none());
    }

    /// **Uma janela não invade o chunk seguinte** — é isso que a torna útil.
    #[test]
    fn a_window_cannot_read_past_its_own_end() {
        let mut r = Reader::new(&[1, 2, 3, 4]);
        let mut w = r.window(2).unwrap();
        assert_eq!(w.u16(), Some(0x0201));
        assert_eq!(w.u16(), None, "a janela tinha 2 bytes");
        assert_eq!(r.u16(), Some(0x0403), "e o leitor de fora avancou 2");
    }

    /// Uma string com bytes inválidos vira texto, não um erro.
    #[test]
    fn a_string_with_broken_utf8_still_reads() {
        let mut bytes = vec![2, 0];
        bytes.extend_from_slice(&[0xFF, 0xFE]);
        let mut r = Reader::new(&bytes);
        assert!(r.string().is_some());
    }

    /// Um comprimento maior que o que resta devolve `None` — sem alocar o que ele pede.
    #[test]
    fn a_lying_string_length_is_none() {
        let mut r = Reader::new(&[0xFF, 0xFF, b'a']);
        assert_eq!(r.string(), None);
    }
}
