//! **A marca do artista** (W9 `docs/Macros/07_plano_W9.md`; W10 `docs/Macros/08_plano_W10_cobertura.md`):
//! por texel, quanto da imagem é tinta que o artista pôs depois da gravação — a cobertura `c` (`0..=CHEIA`) —
//! e o píxel que estava POR BAIXO no 1.º toque (`B`, RGBA8 straight). A gravação a que pertence é a
//! [`MarcaDoArtista::epoca`]. No Play de uma macro o texel mostra `atual + (1 − c)·(palco − B)`
//! (premultiplicado; a tinta é composta «por cima»): `c = 0` é o clipe, `c = CHEIA` a imagem atual. Só os
//! ladrilhos tocados vivem.

use crate::tool::paint::fita::MarcaDaAgua;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// As versões de todas as marcas: duas marcas distintas nunca partilham uma.
static VERSOES: AtomicU64 = AtomicU64::new(1);

fn versao_nova() -> u64 {
    VERSOES.fetch_add(1, Ordering::Relaxed)
}

/// Uma caixa `(x, y, w, h)` em texels.
pub type Caixa = (u32, u32, u32, u32);

/// A cobertura inteira: o texel é só a imagem atual. `u16`: com `u8` o arredondamento da cobertura
/// sozinho chegava a meio nível sobre o palco (W10, a régua dos cruzamentos).
pub const CHEIA: u16 = u16::MAX;

/// O lado de um ladrilho, em texels.
pub const LADO: u32 = 64;
const AREA: usize = (LADO * LADO) as usize;

/// Um ladrilho `LADO`² da marca: a cobertura e o que estava por baixo de cada texel (linha a linha).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ladrilho {
    c: Box<[u16]>,
    b: Box<[[u8; 4]]>,
}

impl Ladrilho {
    fn novo() -> Self {
        Self {
            c: vec![0; AREA].into_boxed_slice(),
            b: vec![[0; 4]; AREA].into_boxed_slice(),
        }
    }

    /// A cobertura do texel `k` do ladrilho.
    #[must_use]
    pub fn c(&self, k: usize) -> u16 {
        self.c[k]
    }

    /// O que estava por baixo do texel `k` (straight; vale com `0 < c < CHEIA`).
    #[must_use]
    pub fn b(&self, k: usize) -> [u8; 4] {
        self.b[k]
    }
}

type Ladrilhos = BTreeMap<(u32, u32), Arc<Ladrilho>>;

/// A marca de uma imagem `w`×`h`.
#[derive(Clone, Debug)]
pub struct MarcaDoArtista {
    w: u32,
    h: u32,
    epoca: u64,
    ladrilhos: Arc<Ladrilhos>,
    versao: u64,
    /// Contém todo texel com `c > 0`; `None` = nenhum.
    caixa: Option<Caixa>,
    /// O estado da água da gravação que a imagem guardou quando ela parou depois do Parar (W14).
    agua_parou: Option<MarcaDaAgua>,
}

/// O que um passo mudou na marca, por ladrilho: como estava e como ficou (`None` = sem ladrilho). O
/// desfazer põe o «antes», o refazer o «depois».
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeltaDaMarca {
    pub epoca: u64,
    pub ladrilhos: Vec<DeltaDoLadrilho>,
}

/// `(ladrilho, antes, depois)`.
pub type DeltaDoLadrilho = ((u32, u32), Option<Arc<Ladrilho>>, Option<Arc<Ladrilho>>);

impl DeltaDaMarca {
    /// O que mudou de `antes` para `depois` (da mesma marca, época `epoca`).
    #[must_use]
    pub fn entre(epoca: u64, antes: &Ladrilhos, depois: &Ladrilhos) -> Self {
        let mut ladrilhos = Vec::new();
        let chaves: std::collections::BTreeSet<&(u32, u32)> =
            antes.keys().chain(depois.keys()).collect();
        for k in chaves {
            let (a, d) = (antes.get(k), depois.get(k));
            let igual = match (a, d) {
                (Some(a), Some(d)) => Arc::ptr_eq(a, d) || a == d,
                (None, None) => true,
                _ => false,
            };
            if !igual {
                ladrilhos.push((*k, a.cloned(), d.cloned()));
            }
        }
        Self { epoca, ladrilhos }
    }

    #[must_use]
    pub fn vazio(&self) -> bool {
        self.ladrilhos.is_empty()
    }

    /// Junta `outro` (da mesma época, posterior) a este: o passo coalescido vai do 1.º antes ao último
    /// depois.
    pub fn junta(&mut self, outro: &Self) {
        if outro.epoca != self.epoca {
            return;
        }
        for (k, a, d) in &outro.ladrilhos {
            match self.ladrilhos.binary_search_by_key(k, |l| l.0) {
                Ok(i) => self.ladrilhos[i].2.clone_from(d),
                Err(i) => self.ladrilhos.insert(i, (*k, a.clone(), d.clone())),
            }
        }
    }

    /// O que este passo segura sozinho: os ladrilhos «depois» (os «antes» são do passo anterior).
    #[must_use]
    pub fn heap_bytes(&self) -> usize {
        let um = AREA * (2 + 4) + std::mem::size_of::<Ladrilho>();
        self.ladrilhos.iter().filter(|l| l.2.is_some()).count() * um
            + self.ladrilhos.capacity() * std::mem::size_of::<DeltaDoLadrilho>()
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

/// A caixa dos texels com `c > 0` em `l` (uma imagem `w`×`h`).
fn caixa_de(l: &Ladrilhos, w: u32, h: u32) -> Option<Caixa> {
    let mut caixa = None;
    for (&(lx, ly), t) in l {
        for (k, &c) in t.c.iter().enumerate() {
            if c == 0 {
                continue;
            }
            let (x, y) = (lx * LADO + k as u32 % LADO, ly * LADO + k as u32 / LADO);
            if x < w && y < h {
                caixa = Some(une(caixa, (x, y, 1, 1)));
            }
        }
    }
    caixa
}

/// `c` composta com a força `a` (`0..=1`): `1 − (1 − c)(1 − a)`, em `0..=CHEIA`.
#[must_use]
pub fn compoe(c: u16, a: f32) -> u16 {
    let resta = f32::from(CHEIA - c) * (1.0 - a.clamp(0.0, 1.0));
    CHEIA - (resta + 0.5) as u16
}

impl MarcaDoArtista {
    /// Uma marca vazia de uma imagem `w`×`h` da gravação `epoca`.
    #[must_use]
    pub fn nova(w: u32, h: u32, epoca: u64) -> Self {
        Self {
            w,
            h,
            epoca,
            ladrilhos: Arc::new(BTreeMap::new()),
            versao: versao_nova(),
            caixa: None,
            agua_parou: None,
        }
    }

    /// **Onde a água da gravação parou na imagem** (W14, `docs/Macros/12_W14_a_agua_depois_do_parar.md`):
    /// a sessão molhada da gravação acabou depois do Parar (o 1.º toque do artista a assa, W12) e a imagem
    /// guardou a água neste estado. `None` = ela corre (ou correu) até assentar.
    #[must_use]
    pub fn agua_da_gravacao_parou(&self) -> Option<MarcaDaAgua> {
        self.agua_parou
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

    /// Contém todo texel com `c > 0` (pode ser maior).
    #[must_use]
    pub fn caixa(&self) -> Option<Caixa> {
        self.caixa
    }

    /// Os ladrilhos (o retrato que um passo guarda para o desfazer).
    #[must_use]
    pub fn ladrilhos(&self) -> &Arc<Ladrilhos> {
        &self.ladrilhos
    }

    /// O ladrilho `(lx, ly)`, se foi tocado.
    #[must_use]
    pub fn ladrilho(&self, lx: u32, ly: u32) -> Option<&Ladrilho> {
        self.ladrilhos.get(&(lx, ly)).map(AsRef::as_ref)
    }

    fn onde(&self, i: usize) -> ((u32, u32), usize) {
        let w = self.w as usize;
        let (x, y) = ((i % w) as u32, (i / w) as u32);
        (
            (x / LADO, y / LADO),
            ((y % LADO) * LADO + x % LADO) as usize,
        )
    }

    /// A cobertura do texel `i` (linha a linha).
    #[must_use]
    pub fn cobertura(&self, i: usize) -> u16 {
        let (l, k) = self.onde(i);
        self.ladrilhos.get(&l).map_or(0, |t| t.c[k])
    }

    /// O que estava por baixo do texel `i` no 1.º toque (vale com `0 < c < CHEIA`).
    #[must_use]
    pub fn por_baixo(&self, i: usize) -> [u8; 4] {
        let (l, k) = self.onde(i);
        self.ladrilhos.get(&l).map_or([0; 4], |t| t.b[k])
    }

    /// O artista tocou o texel `i` (`c > 0`).
    #[must_use]
    pub fn marcado(&self, i: usize) -> bool {
        self.cobertura(i) > 0
    }

    /// Quantos texels o artista tocou.
    #[must_use]
    pub fn quantos(&self) -> usize {
        self.ladrilhos
            .values()
            .map(|t| t.c.iter().filter(|&&c| c > 0).count())
            .sum()
    }

    /// Troca os ladrilhos (de uma marca do mesmo tamanho) e recalcula a caixa.
    pub fn poe_ladrilhos(&mut self, l: Arc<Ladrilhos>) {
        self.caixa = caixa_de(&l, self.w, self.h);
        self.ladrilhos = l;
        self.versao = versao_nova();
    }

    /// **Compõe** cada `(i, a, b)`: o texel `i` recebe tinta com a força `a` (`c ← 1 − (1 − c)(1 − a)`),
    /// e `b` fica como o que estava por baixo se é o 1.º toque. Muda a versão uma vez.
    pub fn compoe_muitos(&mut self, itens: impl IntoIterator<Item = (usize, f32, [u8; 4])>) {
        let n = self.w as usize * self.h as usize;
        let mut caixa = self.caixa;
        let mut mexeu = false;
        let l = Arc::make_mut(&mut self.ladrilhos);
        // O ladrilho em mão sai do mapa e volta quando o seguinte o troca (os itens vêm linha a linha).
        // `bool` = o ladrilho guarda-se (já existia, ou recebeu tinta).
        let mut mao: Option<((u32, u32), Arc<Ladrilho>, bool)> = None;
        let w = self.w as usize;
        for (i, a, b) in itens {
            // `a` NaN cai aqui também.
            if i >= n || a.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
                continue;
            }
            let (x, y) = ((i % w) as u32, (i / w) as u32);
            let chave = (x / LADO, y / LADO);
            if mao.as_ref().is_none_or(|m| m.0 != chave) {
                if let Some((k, t, true)) = mao.take() {
                    l.insert(k, t);
                }
                mao = Some(match l.remove(&chave) {
                    Some(t) => (chave, t, true),
                    None => (chave, Arc::new(Ladrilho::novo()), false),
                });
            }
            let m = mao.as_mut().expect("posto acima");
            let t = Arc::make_mut(&mut m.1);
            let k = ((y % LADO) * LADO + x % LADO) as usize;
            let c = t.c[k];
            let novo = compoe(c, a);
            if novo == c {
                continue;
            }
            if c == 0 {
                t.b[k] = b;
            }
            t.c[k] = novo;
            m.2 = true;
            caixa = Some(une(caixa, (x, y, 1, 1)));
            mexeu = true;
        }
        if let Some((k, t, true)) = mao {
            l.insert(k, t);
        }
        if mexeu {
            self.caixa = caixa;
            self.versao = versao_nova();
        }
    }

    /// **A cobertura viva de uma sessão molhada do artista** (W12): na caixa `(x0, y0, x1, y1)` cada texel
    /// passa a `c0` composta com a cobertura `pa` da sessão (`plano`, `0..=CHEIA`, linha a linha da caixa),
    /// onde `c0` e o `B` são os de `nascimento` (a marca quando a sessão nasceu); onde `c0 = 0` o que está
    /// por baixo é `base(i)` (a tela congelada da sessão). A água anda: o texel que a tinta deixa volta a
    /// `c0`. Muda a versão se mexeu.
    pub fn recompoe_da_agua(
        &mut self,
        nascimento: &Ladrilhos,
        (x0, y0, x1, y1): (u32, u32, u32, u32),
        plano: &[u16],
        base: impl Fn(usize) -> [u8; 4],
    ) {
        let (x1, y1) = (x1.min(self.w), y1.min(self.h));
        let cw = (x1.saturating_sub(x0)) as usize;
        if cw == 0 || y1 <= y0 || plano.len() < cw * (y1 - y0) as usize {
            return;
        }
        let w = self.w as usize;
        let mut caixa = self.caixa;
        let mut mexeu = false;
        let l = Arc::make_mut(&mut self.ladrilhos);
        for ly in y0 / LADO..=(y1 - 1) / LADO {
            for lx in x0 / LADO..=(x1 - 1) / LADO {
                let (tx0, ty0) = ((lx * LADO).max(x0), (ly * LADO).max(y0));
                let (tx1, ty1) = (((lx + 1) * LADO).min(x1), ((ly + 1) * LADO).min(y1));
                let n0 = nascimento.get(&(lx, ly));
                let mut t = l.remove(&(lx, ly));
                let mut guarda = t.is_some();
                for y in ty0..ty1 {
                    for x in tx0..tx1 {
                        let k = ((y % LADO) * LADO + x % LADO) as usize;
                        let pa = plano[(y - y0) as usize * cw + (x - x0) as usize];
                        let (c0, b0) = n0.map_or((0, [0; 4]), |n| (n.c[k], n.b[k]));
                        let c = if pa == 0 {
                            c0
                        } else {
                            compoe(c0, f32::from(pa) / f32::from(CHEIA))
                        };
                        let b = if c0 > 0 {
                            b0
                        } else {
                            base(y as usize * w + x as usize)
                        };
                        let atual = t.as_ref().map_or((0, [0; 4]), |t| (t.c[k], t.b[k]));
                        if atual.0 == c && (c == 0 || atual.1 == b) {
                            continue;
                        }
                        let tt = Arc::make_mut(t.get_or_insert_with(|| Arc::new(Ladrilho::novo())));
                        tt.c[k] = c;
                        tt.b[k] = b;
                        guarda = true;
                        mexeu = true;
                        if c > 0 {
                            caixa = Some(une(caixa, (x, y, 1, 1)));
                        }
                    }
                }
                if let (Some(t), true) = (t, guarda) {
                    l.insert((lx, ly), t);
                }
            }
        }
        if mexeu {
            self.caixa = caixa;
            self.versao = versao_nova();
        }
    }

    /// O que está por baixo de texels já tocados (`(i, B)`, straight) muda — a reescrita da gravação passa
    /// o fim novo por baixo da tinta do artista. Texels intocados ficam.
    pub fn poe_por_baixo(&mut self, itens: impl IntoIterator<Item = (usize, [u8; 4])>) {
        let n = self.w as usize * self.h as usize;
        let mut mexeu = false;
        for (i, b) in itens {
            if i >= n {
                continue;
            }
            let (chave, k) = self.onde(i);
            let l = Arc::make_mut(&mut self.ladrilhos);
            if let Some(t) = l.get_mut(&chave)
                && t.c[k] > 0
                && t.b[k] != b
            {
                Arc::make_mut(t).b[k] = b;
                mexeu = true;
            }
        }
        if mexeu {
            self.versao = versao_nova();
        }
    }

    /// Os texels de `bits` passam a `c = CHEIA` (a força não se sabe: a imagem atual vale).
    pub fn satura_bits(&mut self, bits: &[u64]) {
        let itens = bits.iter().enumerate().flat_map(|(k, &p)| {
            let mut p = p;
            std::iter::from_fn(move || {
                (p != 0).then(|| {
                    let i = k * 64 + p.trailing_zeros() as usize;
                    p &= p - 1;
                    (i, 1.0, [0; 4])
                })
            })
        });
        self.compoe_muitos(itens);
    }

    /// O disco `(cx, cy, r)` (em texels) passa a `c = CHEIA` — o traço em voo, até o passo se registar.
    pub fn marca_disco(&mut self, cx: f32, cy: f32, r: f32) {
        let mut bits = vec![0u64; palavras_para(self.w, self.h)];
        if liga_disco(&mut bits, self.w, self.h, cx, cy, r).is_some() {
            self.satura_bits(&bits);
        }
    }

    /// Põe os ladrilhos de `d` como ficaram (`refaz`) ou como estavam, se ele é desta época.
    pub fn aplica(&mut self, d: &DeltaDaMarca, refaz: bool) {
        if d.epoca != self.epoca || d.vazio() {
            return;
        }
        let l = Arc::make_mut(&mut self.ladrilhos);
        for (k, antes, depois) in &d.ladrilhos {
            match if refaz { depois } else { antes } {
                Some(t) => {
                    l.insert(*k, Arc::clone(t));
                }
                None => {
                    l.remove(k);
                }
            }
        }
        self.caixa = caixa_de(&self.ladrilhos, self.w, self.h);
        self.versao = versao_nova();
    }

    /// **O formato gravado** (`FORMATO` 2): versão, tamanho, época, e cada ladrilho — as coberturas e,
    /// só onde `0 < c < CHEIA`, o que estava por baixo.
    #[must_use]
    pub fn codifica(&self) -> Vec<u8> /* COLOR-RAW-OK: a cobertura e o píxel guardado, bytes do formato */
    {
        let mut out = Vec::new();
        out.push(FORMATO);
        out.extend_from_slice(&self.w.to_le_bytes());
        out.extend_from_slice(&self.h.to_le_bytes());
        out.extend_from_slice(&self.epoca.to_le_bytes());
        out.extend_from_slice(&(self.ladrilhos.len() as u32).to_le_bytes());
        for (&(lx, ly), t) in self.ladrilhos.iter() {
            out.extend_from_slice(&lx.to_le_bytes());
            out.extend_from_slice(&ly.to_le_bytes());
            for c in t.c.iter() {
                out.extend_from_slice(&c.to_le_bytes());
            }
            for (c, b) in t.c.iter().zip(t.b.iter()) {
                if (1..CHEIA).contains(c) {
                    out.extend_from_slice(b);
                }
            }
        }
        out
    }

    /// O inverso de [`Self::codifica`] (e a marca de bits do `FORMATO` 1: cada bit ligado é `c = CHEIA`);
    /// `None` = bytes que não são uma marca.
    #[must_use]
    pub fn descodifica(b: &[u8]) -> Option<Self> {
        let (&versao, b) = b.split_first()?;
        let mut r = Leitor(b);
        let (w, h, epoca) = (r.u32()?, r.u32()?, r.u64()?);
        // Um ficheiro não aloca o que quer: o tecto é o do raster que um ficheiro abre.
        let lado = ph2d_imageio::MAX_RASTER_DIMENSION;
        if w > lado || h > lado {
            return None;
        }
        let mut m = Self::nova(w, h, epoca);
        match versao {
            FORMATO_DOS_BITS => m.satura_bits(&le_os_bits(&mut r, palavras_para(w, h))?),
            FORMATO => {
                let n = r.u32()? as usize;
                if n > (w.div_ceil(LADO) * h.div_ceil(LADO)) as usize {
                    return None;
                }
                let mut l = BTreeMap::new();
                for _ in 0..n {
                    let (lx, ly) = (r.u32()?, r.u32()?);
                    if lx >= w.div_ceil(LADO) || ly >= h.div_ceil(LADO) {
                        return None;
                    }
                    let mut t = Ladrilho::novo();
                    for (c, b) in t.c.iter_mut().zip(r.bytes(AREA * 2)?.as_chunks::<2>().0) {
                        *c = u16::from_le_bytes(*b);
                    }
                    for k in 0..AREA {
                        if (1..CHEIA).contains(&t.c[k]) {
                            t.b[k].copy_from_slice(r.bytes(4)?);
                        }
                    }
                    l.insert((lx, ly), Arc::new(t));
                }
                if !r.0.is_empty() {
                    return None;
                }
                m.poe_ladrilhos(Arc::new(l));
            }
            _ => return None,
        }
        Some(m)
    }
}

/// As palavras do `FORMATO` 1 (corridas de zeros contadas).
fn le_os_bits(r: &mut Leitor<'_>, n: usize) -> Option<Vec<u64>> {
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
    Some(p)
}

/// A versão do formato de [`MarcaDoArtista::codifica`] (W10: a cobertura).
const FORMATO: u8 = 2;
/// A da W9 (um bit por texel), que ainda se lê.
const FORMATO_DOS_BITS: u8 = 1;

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

    fn bytes(&mut self, n: usize) -> Option<&[u8]> {
        let (a, b) = self.0.split_at_checked(n)?;
        self.0 = b;
        Some(a)
    }
}

#[cfg(test)]
#[path = "marca_tests.rs"]
mod tests;
