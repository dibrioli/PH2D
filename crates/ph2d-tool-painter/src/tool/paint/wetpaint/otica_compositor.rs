//! O fio do COMPOSITOR da óptica (filho de [`super`] por LOC, B52.R8): compõe sobre o espelho e recorta as caixas.

use super::{CaixaCel, CaixasDaImagem, Imagem, RespostaDaOtica, VeuDaCaixa};
use crate::tool::paint::wetpaint::composite_otica::{
    CaixasDaOtica, EntradaDaOtica, PlanosDaOtica, compoe_a_otica,
};
use ph2d_wet_paint::painter::espelho::Espelho;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

/// O que o worker manda ao compositor.
pub(super) struct Trabalho {
    pub(super) espelho: Box<Espelho>,
    /// `trabalho.caixa` = a envolvente das `faixas`.
    pub(super) entrada: EntradaDaOtica,
    /// O sujo em pedaços, cada um preso à grade (B14.1).
    pub(super) faixas: Vec<CaixaCel>,
    pub(super) epoca: u64,
    pub(super) passos: u64,
    /// A camada activa, se o pedido quer o véu.
    pub(super) veu: Option<usize>,
}

/// O laço do compositor: até o worker fechar o canal de trabalho.
pub(super) fn compositor(rx: &Receiver<Trabalho>, volta: &Sender<Box<Espelho>>, ui: &Sender<RespostaDaOtica>) {
    let mut planos = PlanosDaOtica::default();
    while let Ok(t) = rx.recv() {
        let t0 = Instant::now();
        let (espelho, imagem) = no_pool(|| compoe(t, &mut planos));
        crate::wet_diag::note_otica(t0.elapsed().as_secs_f32() * 1000.0);
        // O espelho volta primeiro: o worker pode copiar a próxima enquanto a tela aplica esta.
        let _ = volta.send(espelho);
        let _ = ui.send(RespostaDaOtica::Imagem(Box::new(imagem)));
    }
}

/// Corre `f` DENTRO do pool do rayon, pela razão do `PainterTool::no_pool` (`por_quadro.rs`): os passes paralelos
/// lançados de fora do pool entram pela fila global e adormecem quem chama no trinco.
fn no_pool<R: Send>(f: impl FnOnce() -> R + Send) -> R {
    if rayon::current_thread_index().is_some()
        || !crate::wet_diag::NO_POOL.load(std::sync::atomic::Ordering::Relaxed)
    {
        return f();
    }
    rayon::join(f, || ()).0
}

/// A metade de cima sobre o espelho, e a imagem tirada dos planos de trabalho.
fn compoe(t: Trabalho, planos: &mut PlanosDaOtica) -> (Box<Espelho>, Imagem) {
    let (gw, gh) = t.entrada.grade;
    if planos.pigment.len() != gw * gh * 4 {
        *planos = PlanosDaOtica::da_grade((gw, gh));
    }
    let layers = t.espelho.camadas_render();
    let faixas = crate::tool::paint::wetpaint::composite_otica::faixas_da_otica(&t.entrada, &layers, &t.faixas);
    let oleo = t.entrada.factos.oleo_da_tela(&layers);
    let mut caixas = Vec::with_capacity(faixas.len());
    let mut entrada = t.entrada;
    for f in faixas {
        if let Some(tr) = entrada.trabalho.as_mut() {
            tr.caixa = f;
        }
        // Sem cache: a chave leva a `revisao`, que anda a cada passo (J13 §2); o campo é exacto sem ela.
        planos.campo_cache = None;
        let saida = compoe_a_otica(&entrada, &layers, planos);
        caixas.extend(saida.caixas.map(|c| {
            let veu = t
                .veu
                .and_then(|i| t.espelho.camadas.get(i))
                .map(|c_| veu_da_caixa(&c_.grid, c.caixa_px, entrada.ratio, (gw, gh)));
            caixas_da_imagem(f, c, planos, (gw, gh), veu)
        }));
    }
    planos.campo_cache = None;
    drop(layers);
    let imagem = Imagem {
        oleo,
        marca: entrada.marca,
        epoca: t.epoca,
        passos: t.passos,
        caixas,
    };
    (t.espelho, imagem)
}

fn caixas_da_imagem(
    sujo: CaixaCel,
    c: CaixasDaOtica,
    planos: &PlanosDaOtica,
    (gw, gh): (usize, usize),
    veu: Option<VeuDaCaixa>,
) -> CaixasDaImagem {
    let corta = |plano: &[u8]| (plano.len() == gw * gh).then(|| recorta(plano, 1, gw, c.caixa_cel));
    CaixasDaImagem {
        sujo,
        caixa_cel: c.caixa_cel,
        caixa_px: c.caixa_px,
        pigmento: recorta(&planos.pigment, 4, gw, c.caixa_cel),
        sombra: corta(&planos.sombra),
        grao: corta(&planos.grao),
        suave: (planos.suave.len() == gw * gh)
            .then(|| recorta(&planos.suave, 1, gw, c.caixa_suave)),
        caixa_suave: c.caixa_suave,
        sem_oleo: c.sem_oleo,
        veu,
    }
}

/// As linhas de células que a caixa de píxeis toca, com uma de folga de cada lado.
fn veu_da_caixa(
    g: &ph2d_wet_paint::grid::Grid,
    (_, py0, _, py1): (usize, usize, usize, usize),
    ratio: u8,
    (_, gh): (usize, usize),
) -> VeuDaCaixa {
    let rr = usize::from(ratio).max(1);
    let lo = (py0 / rr + 1).min(gh);
    let hi = (py1.saturating_sub(1) / rr + 1).min(gh);
    let (a, b) = (lo - 1, (hi + 1).min(g.rows - 1));
    VeuDaCaixa {
        linha0: a,
        s: g.s,
        film: g.film[a * g.s..(b + 1) * g.s].to_vec(),
        wet: g.wet[a * g.s..(b + 1) * g.s].to_vec(),
    }
}

/// Os bytes da caixa de um plano da grade (`bpp` bytes por célula, linha `gw`).
fn recorta(plano: &[u8], bpp: usize, gw: usize, (x0, y0, x1, y1): CaixaCel) -> Vec<u8> {
    let mut v = Vec::with_capacity((x1 + 1 - x0) * (y1 + 1 - y0) * bpp);
    for cy in y0..=y1 {
        let o = (cy - 1) * gw * bpp;
        v.extend_from_slice(&plano[o + (x0 - 1) * bpp..o + x1 * bpp]);
    }
    v
}

/// O inverso de [`recorta`].
pub(super) fn cola(plano: &mut [u8], bpp: usize, gw: usize, (x0, y0, x1, y1): CaixaCel, b: &[u8]) {
    let cw = (x1 + 1 - x0) * bpp;
    for (k, cy) in (y0..=y1).enumerate() {
        let o = (cy - 1) * gw * bpp + (x0 - 1) * bpp;
        plano[o..o + cw].copy_from_slice(&b[k * cw..(k + 1) * cw]);
    }
}
