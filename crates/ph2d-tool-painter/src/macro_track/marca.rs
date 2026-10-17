//! **A marca do artista** (W9, `docs/Macros/07_plano_W9.md`; decisão do dono em
//! `docs/Macros/06_auditoria_W8_record.md`): por texel, «o artista escreveu aqui depois da gravação».
//! Um bit por texel; a gravação a que pertence é a [`MarcaDoArtista::epoca`]. No Play de uma macro, um
//! texel marcado mostra a imagem atual e os outros o clipe.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// As versões de todas as marcas: duas marcas distintas nunca partilham uma.
static VERSOES: AtomicU64 = AtomicU64::new(1);

fn versao_nova() -> u64 {
    VERSOES.fetch_add(1, Ordering::Relaxed)
}

/// Uma caixa `(x, y, w, h)` em texels.
pub type Caixa = (u32, u32, u32, u32);

/// A marca de uma imagem `w`×`h`.
#[derive(Clone, Debug)]
pub struct MarcaDoArtista {
    w: u32,
    h: u32,
    epoca: u64,
    palavras: Arc<Vec<u64>>,
    versao: u64,
    /// Contém todo texel marcado (pode ser maior); `None` = nenhum.
    caixa: Option<Caixa>,
}

/// Os bits que um passo LIGOU numa marca (o desfazer desliga-os; o refazer liga-os de novo).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeltaDaMarca {
    pub epoca: u64,
    /// `(índice da palavra, bits ligados)`.
    pub palavras: Vec<(u32, u64)>,
}

impl DeltaDaMarca {
    /// Junta `outro` (da mesma época) a este: o passo coalescido liga os dois.
    pub fn junta(&mut self, outro: &Self) {
        if outro.epoca != self.epoca {
            return;
        }
        for &(i, b) in &outro.palavras {
            match self.palavras.binary_search_by_key(&i, |p| p.0) {
                Ok(k) => self.palavras[k].1 |= b,
                Err(k) => self.palavras.insert(k, (i, b)),
            }
        }
    }

    #[must_use]
    pub fn heap_bytes(&self) -> usize {
        self.palavras.capacity() * std::mem::size_of::<(u32, u64)>()
    }
}

/// Quantas palavras de 64 bits cobrem `n` texels.
#[must_use]
pub fn palavras_para(w: u32, h: u32) -> usize {
    (w as usize * h as usize).div_ceil(64)
}

/// Liga o bit `i` de `p`.
pub fn liga(p: &mut [u64], i: usize) {
    p[i / 64] |= 1 << (i % 64);
}

/// O bit `i` de `p`.
#[must_use]
pub fn ligado(p: &[u64], i: usize) -> bool {
    p[i / 64] >> (i % 64) & 1 == 1
}

/// Liga em `p` (uma tela `w`×`h`) os texels cujo centro cai no disco `(cx, cy, r)`, e devolve a caixa
/// que ele cobriu (`None` = fora da tela).
pub fn liga_disco(p: &mut [u64], w: u32, h: u32, cx: f32, cy: f32, r: f32) -> Option<Caixa> {
    if !(cx.is_finite() && cy.is_finite() && r.is_finite()) || r <= 0.0 {
        return None;
    }
    let (wf, hf) = (w as f32, h as f32);
    let x0 = (cx - r).floor().max(0.0);
    let y0 = (cy - r).floor().max(0.0);
    let x1 = (cx + r).ceil().min(wf);
    let y1 = (cy + r).ceil().min(hf);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let (x0, y0, x1, y1) = (x0 as u32, y0 as u32, x1 as u32, y1 as u32);
    let r2 = r * r;
    let mut caixa: Option<Caixa> = None;
    for y in y0..y1 {
        let dy = y as f32 + 0.5 - cy;
        for x in x0..x1 {
            let dx = x as f32 + 0.5 - cx;
            if dx * dx + dy * dy <= r2 {
                liga(p, (y * w + x) as usize);
                caixa = Some(une(caixa, (x, y, 1, 1)));
            }
        }
    }
    caixa
}

/// A caixa que contém `a` e `b`.
#[must_use]
pub fn une(a: Option<Caixa>, b: Caixa) -> Caixa {
    let Some(a) = a else { return b };
    let x0 = a.0.min(b.0);
    let y0 = a.1.min(b.1);
    let x1 = (a.0 + a.2).max(b.0 + b.2);
    let y1 = (a.1 + a.3).max(b.1 + b.3);
    (x0, y0, x1 - x0, y1 - y0)
}

/// A caixa dos bits ligados de `p` (uma tela de largura `w`).
fn caixa_de(p: &[u64], w: u32, h: u32) -> Option<Caixa> {
    let n = w as usize * h as usize;
    let mut caixa = None;
    for (k, &b) in p.iter().enumerate() {
        let mut b = b;
        while b != 0 {
            let i = k * 64 + b.trailing_zeros() as usize;
            b &= b - 1;
            if i < n {
                let (x, y) = ((i % w as usize) as u32, (i / w as usize) as u32);
                caixa = Some(une(caixa, (x, y, 1, 1)));
            }
        }
    }
    caixa
}

impl MarcaDoArtista {
    /// Uma marca vazia de uma imagem `w`×`h` da gravação `epoca`.
    #[must_use]
    pub fn nova(w: u32, h: u32, epoca: u64) -> Self {
        Self {
            w,
            h,
            epoca,
            palavras: Arc::new(vec![0; palavras_para(w, h)]),
            versao: versao_nova(),
            caixa: None,
        }
    }

    #[must_use]
    pub fn tamanho(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    #[must_use]
    pub fn epoca(&self) -> u64 {
        self.epoca
    }

    /// Muda a cada escrita: a chave de quem mostra a marca.
    #[must_use]
    pub fn versao(&self) -> u64 {
        self.versao
    }

    #[must_use]
    pub fn vazia(&self) -> bool {
        self.caixa.is_none()
    }

    /// Contém todo texel marcado (pode ser maior).
    #[must_use]
    pub fn caixa(&self) -> Option<Caixa> {
        self.caixa
    }

    #[must_use]
    pub fn palavras(&self) -> &Arc<Vec<u64>> {
        &self.palavras
    }

    /// O texel `i` (linha a linha) está marcado.
    #[must_use]
    pub fn marcado(&self, i: usize) -> bool {
        ligado(&self.palavras, i)
    }

    /// Quantos texels estão marcados.
    #[must_use]
    pub fn quantos(&self) -> usize {
        self.palavras.iter().map(|b| b.count_ones() as usize).sum()
    }

    /// Troca as palavras (do mesmo tamanho) e recalcula a caixa.
    ///
    /// # Panics
    /// Com outro número de palavras (um erro de quem chama).
    pub fn poe_palavras(&mut self, palavras: Arc<Vec<u64>>) {
        assert_eq!(palavras.len(), self.palavras.len(), "outro tamanho");
        self.caixa = caixa_de(&palavras, self.w, self.h);
        self.palavras = palavras;
        self.versao = versao_nova();
    }

    /// Liga o disco `(cx, cy, r)` (em texels).
    pub fn marca_disco(&mut self, cx: f32, cy: f32, r: f32) {
        let (w, h) = (self.w, self.h);
        let p: &mut Vec<u64> = Arc::make_mut(&mut self.palavras);
        if let Some(c) = liga_disco(p, w, h, cx, cy, r) {
            self.caixa = Some(une(self.caixa, c));
            self.versao = versao_nova();
        }
    }

    /// Liga (`liga == true`) ou desliga os bits de `d`, se ele é desta época.
    pub fn aplica(&mut self, d: &DeltaDaMarca, liga: bool) {
        if d.epoca != self.epoca || d.palavras.is_empty() {
            return;
        }
        let p = Arc::make_mut(&mut self.palavras);
        for &(i, b) in &d.palavras {
            if let Some(w) = p.get_mut(i as usize) {
                if liga {
                    *w |= b;
                } else {
                    *w &= !b;
                }
            }
        }
        self.caixa = caixa_de(&self.palavras, self.w, self.h);
        self.versao = versao_nova();
    }

    /// A marca une `bits` (do mesmo tamanho): devolve o que ficou ligado de novo.
    pub fn une_bits(&mut self, bits: &[u64]) -> DeltaDaMarca {
        let mut d = DeltaDaMarca {
            epoca: self.epoca,
            palavras: Vec::new(),
        };
        for (i, (&a, &b)) in self.palavras.iter().zip(bits).enumerate() {
            let novos = b & !a;
            if novos != 0 {
                d.palavras.push((i as u32, novos));
            }
        }
        if !d.palavras.is_empty() {
            self.aplica(&d, true);
        }
        d
    }

    /// **O formato gravado**: versão, tamanho, época e as palavras por corridas (zeros contados).
    #[must_use]
    pub fn codifica(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(FORMATO);
        out.extend_from_slice(&self.w.to_le_bytes());
        out.extend_from_slice(&self.h.to_le_bytes());
        out.extend_from_slice(&self.epoca.to_le_bytes());
        let p = &self.palavras;
        let mut k = 0;
        while k < p.len() {
            let zeros = p[k..].iter().take_while(|&&b| b == 0).count();
            k += zeros;
            let cheias = p[k..].iter().take_while(|&&b| b != 0).count();
            out.extend_from_slice(&(zeros as u32).to_le_bytes());
            out.extend_from_slice(&(cheias as u32).to_le_bytes());
            for &b in &p[k..k + cheias] {
                out.extend_from_slice(&b.to_le_bytes());
            }
            k += cheias;
        }
        out
    }

    /// O inverso de [`Self::codifica`]; `None` = bytes que não são uma marca deste formato.
    #[must_use]
    pub fn descodifica(b: &[u8]) -> Option<Self> {
        let (&versao, b) = b.split_first()?;
        if versao != FORMATO {
            return None;
        }
        let mut r = Leitor(b);
        let (w, h, epoca) = (r.u32()?, r.u32()?, r.u64()?);
        // Um ficheiro não aloca o que quer: o tecto é o do raster que um ficheiro abre.
        let lado = ph2d_imageio::MAX_RASTER_DIMENSION;
        if w > lado || h > lado {
            return None;
        }
        let n = palavras_para(w, h);
        let mut p = Vec::new();
        while !r.0.is_empty() {
            let (zeros, cheias) = (r.u32()? as usize, r.u32()? as usize);
            if p.len() + zeros + cheias > n {
                return None;
            }
            p.resize(p.len() + zeros, 0);
            for _ in 0..cheias {
                p.push(r.u64()?);
            }
        }
        p.resize(n, 0);
        let caixa = caixa_de(&p, w, h);
        Some(Self {
            w,
            h,
            epoca,
            palavras: Arc::new(p),
            versao: versao_nova(),
            caixa,
        })
    }
}

/// A versão do formato de [`MarcaDoArtista::codifica`].
const FORMATO: u8 = 1;

/// FNV-1a de 64 bits (Fowler–Noll–Vo, os parâmetros publicados), continuando de `h`.
fn fnv1a(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

const FNV_INICIO: u64 = 0xcbf2_9ce4_8422_2325;

/// **A época de uma gravação**: o hash ESTÁVEL (gravado no projeto) da gravação CRUA — o estado
/// inicial e os eventos; as edições das etapas não a mudam. Gravar de novo = outra época = outra marca.
#[must_use]
pub fn epoca_da_gravacao(t: &ph2d_macro::Track) -> u64 {
    let mut h = fnv1a(FNV_INICIO, &(t.initial.len() as u64).to_le_bytes());
    h = fnv1a(h, &t.initial);
    for e in &t.events {
        h = fnv1a(h, &e.t.0.to_le_bytes());
        h = fnv1a(h, &(e.payload.len() as u64).to_le_bytes());
        h = fnv1a(h, &e.payload);
    }
    h
}

struct Leitor<'a>(&'a [u8]);

impl Leitor<'_> {
    fn u32(&mut self) -> Option<u32> {
        let (a, b) = self.0.split_first_chunk::<4>()?;
        self.0 = b;
        Some(u32::from_le_bytes(*a))
    }

    fn u64(&mut self) -> Option<u64> {
        let (a, b) = self.0.split_first_chunk::<8>()?;
        self.0 = b;
        Some(u64::from_le_bytes(*a))
    }
}

#[cfg(test)]
#[path = "marca_tests.rs"]
mod tests;
