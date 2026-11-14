//! O laço de píxel no fio do COMPOSITOR (B11.3, filho de [`super`] por LOC): o pedido leva a entrada do píxel
//! ([`EntradaDoPixel`]) e o compositor devolve os RGBA da caixa; o fio da tela só os copia. Só sem Óleo Real (o grão
//! do papel empresta o `paper_image`), sem vidro na camada e sem a marca do artista: fora disso, o laço de casa.

use super::super::super::adormecida::Congelada;
use super::super::super::composite::pixel::{LacoDePixel, laco_de_pixel};
use super::super::super::composite_otica::{CaixasDaOtica, PlanosDaOtica};
use super::super::super::grid_map::SampleU;
use super::super::super::smooth_edges::Suaves;
use super::{CaixaCel, FormaDaTela};
use crate::tool::paint::watercolor_accum::SplatGate;
use std::sync::Arc;

/// O que o laço de píxel lê da ferramenta, partilhado por `Arc` (o mesmo que [`super::super::super::composite`] lê).
#[derive(Clone)]
pub(in crate::tool::paint::wetpaint) struct EntradaDoPixel {
    pub(in crate::tool::paint::wetpaint) base: Arc<Vec<u8>>,
    /// Selecção, protecção, alfa preso.
    pub(in crate::tool::paint::wetpaint) gates: (SplatGate, SplatGate, SplatGate),
    /// Já filtrada pelo véu (com `show_wet` o laço não a lê).
    pub(in crate::tool::paint::wetpaint) congelada: Option<Arc<Congelada>>,
    pub(in crate::tool::paint::wetpaint) modo: ph2d_painter_brush::BrushBlend,
    pub(in crate::tool::paint::wetpaint) vazio: bool,
    pub(in crate::tool::paint::wetpaint) linhas: Option<bool>,
}

fn mesmo<T>(a: Option<&Arc<T>>, b: Option<&Arc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

impl EntradaDoPixel {
    /// A mesma entrada (os mesmos `Arc`, os mesmos valores): os RGBA do compositor são os que o laço de casa daria.
    pub(in crate::tool::paint::wetpaint) fn mesma(&self, o: &Self) -> bool {
        Arc::ptr_eq(&self.base, &o.base)
            && mesmo(self.gates.0.as_ref(), o.gates.0.as_ref())
            && mesmo(self.gates.1.as_ref(), o.gates.1.as_ref())
            && mesmo(self.gates.2.as_ref(), o.gates.2.as_ref())
            && mesmo(self.congelada.as_ref(), o.congelada.as_ref())
            && self.modo == o.modo
            && self.vazio == o.vazio
            && self.linhas == o.linhas
    }
}

/// Os RGBA da `caixa_px` (`(px1 - px0) * 4` bytes por linha), pelo mesmo [`laco_de_pixel`] da casa sobre os planos do
/// compositor. `linhas` = o rascunho de linhas inteiras (reusado entre imagens).
pub(in crate::tool::paint::wetpaint) fn rgba_da_caixa(
    e: &EntradaDoPixel,
    planos: &PlanosDaOtica,
    forma: FormaDaTela,
    c: &CaixasDaOtica,
    linhas: &mut Vec<u8>,
) -> Vec<u8> {
    let (w, h) = forma.tela;
    let (gw, gh) = forma.grade;
    let (px0, py0, px1, py1) = c.caixa_px;
    let stride = w * 4;
    let rr = usize::from(forma.ratio);
    let sampler = SampleU::new(forma.ratio, gw, gh);
    // Sem Óleo Real (a única entrada que vem aqui): a orla do Smooth Edges é sempre a da cor.
    let orla = true;
    let suavizada = Suaves::nova(
        sampler,
        (&planos.pigment, stride),
        (&planos.suave, gw, gh),
        (w, h),
        c.caixa_px,
        rr,
        e.linhas,
        orla,
    );
    let (gate_sel, gate_prot, gate_alock) = &e.gates;
    let laco = LacoDePixel {
        w,
        caixa_px: c.caixa_px,
        grade: (gw, gh),
        rr,
        sampler,
        pigment: &planos.pigment,
        base: &e.base,
        sombra: &planos.sombra,
        grao_cel: &planos.grao,
        congelada: e.congelada.as_deref(),
        suavizada: suavizada.as_ref(),
        sem_oleo: &c.sem_oleo,
        orla,
        grao: None,
        gsel: gate_sel.as_deref().map(Vec::as_slice),
        gprot: gate_prot.as_deref().map(Vec::as_slice),
        alock_on: gate_alock.is_some(),
        modo: e.modo,
        vazio: e.vazio,
        exata: false,
        vbase: None,
    };
    let n = py1 - py0;
    // Sem vidro, o laço escreve TODA a `px0..px1` de cada linha e não lê o que lá estava: o rascunho não se limpa.
    linhas.resize(n * stride, 0);
    let (vl, cl) = (
        (0..n).map(|_| None).collect(),
        (0..n).map(|_| None).collect(),
    );
    laco_de_pixel(&laco, linhas, vl, cl);
    let cw = (px1 - px0) * 4;
    let mut rgba = Vec::with_capacity(n * cw);
    for row in linhas.chunks_exact(stride) {
        rgba.extend_from_slice(&row[px0 * 4..px0 * 4 + cw]);
    }
    rgba
}

/// **O que o laço de píxel LEU dos planos do compositor** (B11.3): a janela de células que ele pode ler à volta da
/// `caixa_px` (o alcance do [`Suaves`] e um vizinho do amostrador) e os bytes de cada plano nela. Os planos do
/// compositor fora das caixas dele não são os da tela (a casa compõe dabs que ele não vê; uma faixa recusada fica só
/// nele): o fio da tela usa os RGBA só se os SEUS planos na janela são estes ao byte — senão o laço é o de casa.
pub(in crate::tool::paint::wetpaint) struct Lido {
    janela: CaixaCel,
    /// `pigment` · `suave` · `sombra` · `grao`; `None` = o plano vazio.
    planos: [Option<Vec<u8>>; 4],
}

const BPP: [usize; 4] = [4, 1, 1, 1];

fn planos_de(p: &PlanosDaOtica) -> [&[u8]; 4] {
    [&p.pigment, &p.suave, &p.sombra, &p.grao]
}

impl Lido {
    /// A janela de leitura da `caixa_px` (1-based, presa à grade) e os planos nela.
    pub(in crate::tool::paint::wetpaint) fn de(
        planos: &PlanosDaOtica,
        forma: FormaDaTela,
        (px0, py0, px1, py1): (usize, usize, usize, usize),
    ) -> Self {
        let (gw, gh) = forma.grade;
        let rr = usize::from(forma.ratio).max(1);
        let a =
            super::super::super::smooth_edges::alcance(super::super::super::smooth_edges::RAIO_MAX);
        let lo = |p: usize| (p.saturating_sub(a) / rr).saturating_sub(2) + 1;
        let hi = |p: usize, n: usize| ((p + a) / rr + 2).min(n - 1) + 1;
        let janela = (lo(px0), lo(py0), hi(px1, gw), hi(py1, gh));
        let planos = planos_de(planos)
            .into_iter()
            .zip(BPP)
            .map(|(p, bpp)| (p.len() == gw * gh * bpp).then(|| recorta(p, bpp, gw, janela)))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| unreachable!("quatro planos"));
        Self { janela, planos }
    }

    /// Os planos da tela na janela são os que o compositor leu, ao byte (o vazio também)?
    pub(in crate::tool::paint::wetpaint) fn confere(
        &self,
        planos: &PlanosDaOtica,
        (gw, gh): (usize, usize),
    ) -> bool {
        planos_de(planos)
            .into_iter()
            .zip(BPP)
            .zip(&self.planos)
            .all(|((p, bpp), lido)| match lido {
                None => p.len() != gw * gh * bpp,
                Some(b) => p.len() == gw * gh * bpp && igual(p, bpp, gw, self.janela, b),
            })
    }
}

fn recorta(plano: &[u8], bpp: usize, gw: usize, (x0, y0, x1, y1): CaixaCel) -> Vec<u8> {
    let mut v = Vec::with_capacity((x1 + 1 - x0) * (y1 + 1 - y0) * bpp);
    for cy in y0..=y1 {
        let o = (cy - 1) * gw * bpp;
        v.extend_from_slice(&plano[o + (x0 - 1) * bpp..o + x1 * bpp]);
    }
    v
}

fn igual(plano: &[u8], bpp: usize, gw: usize, (x0, y0, x1, y1): CaixaCel, b: &[u8]) -> bool {
    let cw = (x1 + 1 - x0) * bpp;
    (y0..=y1).zip(b.chunks_exact(cw)).all(|(cy, linha)| {
        let o = (cy - 1) * gw * bpp + (x0 - 1) * bpp;
        plano[o..o + cw] == *linha
    })
}

/// Cola os RGBA da caixa nas linhas `py0..py1` da tela (o inverso do recorte de [`rgba_da_caixa`]).
pub(in crate::tool::paint::wetpaint) fn cola_na_tela(
    tela: &mut [u8],
    w: usize,
    (px0, py0, px1, py1): (usize, usize, usize, usize),
    rgba: &[u8],
) {
    let (stride, cw) = (w * 4, (px1 - px0) * 4);
    for (k, y) in (py0..py1).enumerate() {
        let o = y * stride + px0 * 4;
        tela[o..o + cw].copy_from_slice(&rgba[k * cw..(k + 1) * cw]);
    }
}
