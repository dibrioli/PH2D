//! O fio do COMPOSITOR da óptica (filho de [`super`] por LOC, B52.R8): compõe sobre o espelho e recorta as caixas.

use super::pixel::{EntradaDoPixel, Lido, rgba_da_caixa};
use super::{CaixaCel, CaixasDaImagem, FormaDaTela, Imagem, RespostaDaOtica, VeuDaCaixa};
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
    /// B11.3: a entrada do laço de píxel (`None` = a tela compõe-no em casa).
    pub(super) pixel: Option<EntradaDoPixel>,
    /// A porta de teste de B11.A1 ([`super::DoFioPrincipal::mata_no_teste`]).
    #[cfg(test)]
    pub(super) panica: bool,
}

/// O laço do compositor: até o worker fechar o canal de trabalho.
pub(super) fn compositor(
    rx: &Receiver<Trabalho>,
    volta: &Sender<Box<Espelho>>,
    ui: &Sender<RespostaDaOtica>,
) {
    let mut planos = PlanosDaOtica::default();
    let mut linhas = Vec::new();
    while let Ok(t) = rx.recv() {
        let t0 = Instant::now();
        let (espelho, imagem) = no_pool(|| compoe(t, &mut planos, &mut linhas));
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
fn compoe(t: Trabalho, planos: &mut PlanosDaOtica, linhas: &mut Vec<u8>) -> (Box<Espelho>, Imagem) {
    #[cfg(test)]
    assert!(!t.panica, "compoe em panico (porta de teste do B11.A1)");
    let (gw, gh) = t.entrada.grade;
    if planos.pigment.len() != gw * gh * 4 {
        *planos = PlanosDaOtica::da_grade((gw, gh));
    }
    let layers = t.espelho.camadas_render();
    let faixas = crate::tool::paint::wetpaint::composite_otica::faixas_da_otica(
        &t.entrada, &layers, &t.faixas,
    );
    let oleo = t.entrada.factos.oleo_da_tela(&layers);
    let mut caixas = Vec::with_capacity(faixas.len());
    let mut entrada = t.entrada;
    let forma = FormaDaTela {
        ratio: entrada.ratio,
        grade: entrada.grade,
        tela: entrada.tela,
    };
    // Sem Óleo Real (o grão do papel é da ferramenta). Os RGBA compõem-se depois de TODAS as faixas: o laço lê a
    // vizinhança da caixa, e a tela, quando os copia, já colou as faixas todas.
    let pixel = t.pixel.filter(|_| oleo.is_none());
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
    let caixas = caixas
        .into_iter()
        .map(|mut cx: CaixasDaImagem| {
            if let Some(e) = pixel.as_ref() {
                let c = CaixasDaOtica {
                    caixa_cel: cx.caixa_cel,
                    caixa_px: cx.caixa_px,
                    caixa_suave: cx.caixa_suave,
                    sem_oleo: cx.sem_oleo.take(),
                };
                cx.rgba = Some(rgba_da_caixa(e, planos, forma, &c, linhas));
                cx.lido = Some(Lido::de(planos, forma, c.caixa_px));
                cx.sem_oleo = c.sem_oleo;
            }
            cx
        })
        .collect();
    planos.campo_cache = None;
    drop(layers);
    let imagem = Imagem {
        oleo,
        marca: entrada.marca,
        epoca: t.epoca,
        passos: t.passos,
        caixas,
        pixel,
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
        rgba: None,
        lido: None,
    }
}

/// As células que a caixa de píxeis toca, com uma de folga de cada lado (B11.A2: só as colunas dela).
pub(super) fn veu_da_caixa(
    g: &ph2d_wet_paint::grid::Grid,
    (px0, py0, px1, py1): (usize, usize, usize, usize),
    ratio: u8,
    (gw, gh): (usize, usize),
) -> VeuDaCaixa {
    let rr = usize::from(ratio).max(1);
    let celulas = |p0: usize, p1: usize, n: usize, fim: usize| {
        let (lo, hi) = ((p0 / rr + 1).min(n), (p1.saturating_sub(1) / rr + 1).min(n));
        (lo - 1, (hi + 1).min(fim))
    };
    let (a, b) = celulas(py0, py1, gh, g.rows - 1);
    let (c0, c1) = celulas(px0, px1, gw, g.s - 1);
    let largura = c1 + 1 - c0;
    let mut film = Vec::with_capacity((b + 1 - a) * largura);
    let mut wet = Vec::with_capacity((b + 1 - a) * largura);
    for y in a..=b {
        let o = y * g.s;
        film.extend_from_slice(&g.film[o + c0..=o + c1]);
        wet.extend_from_slice(&g.wet[o + c0..=o + c1]);
    }
    VeuDaCaixa {
        linha0: a,
        col0: c0,
        s: largura,
        film,
        wet,
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

/// A caixa a partir da qual a [`cola`] vai por faixas no pool (ADR-0109, o mesmo laço por linha da composição).
/// Recurso: a largura de banda da memória (a caixa que já não cabe na cache) contra o custo de acordar o pool;
/// `mede_cola_por_faixa`, smoke, mín 9: 1 MB 0,6× · 4 MB 0,9–1,3× (o empate) · 33 MB 1,3× (tabela no BACKLOG B11.4).
const COLA_EM_FAIXAS_BYTES: usize = 1 << 22;

/// O inverso de [`recorta`] (B11.4): numa caixa grande as linhas copiam-se no pool — cada uma escreve só a sua,
/// logo o mesmo byte em qualquer número de fios.
pub(super) fn cola(plano: &mut [u8], bpp: usize, gw: usize, caixa: CaixaCel, b: &[u8]) {
    if b.len() < COLA_EM_FAIXAS_BYTES {
        return cola_serial(plano, bpp, gw, caixa, b);
    }
    cola_em_faixas(plano, bpp, gw, caixa, b);
}

fn cola_em_faixas(plano: &mut [u8], bpp: usize, gw: usize, (x0, y0, x1, y1): CaixaCel, b: &[u8]) {
    use rayon::prelude::*;
    let (cw, linha) = ((x1 + 1 - x0) * bpp, gw * bpp);
    let col = (x0 - 1) * bpp;
    plano[(y0 - 1) * linha..y1 * linha]
        .par_chunks_mut(linha)
        .zip(b.par_chunks(cw))
        .for_each(|(dst, src)| dst[col..col + cw].copy_from_slice(src));
}

/// A cola de uma linha de cada vez (a de antes do B11.4, verbatim): caixa pequena e o oráculo do teste.
fn cola_serial(plano: &mut [u8], bpp: usize, gw: usize, (x0, y0, x1, y1): CaixaCel, b: &[u8]) {
    let cw = (x1 + 1 - x0) * bpp;
    for (k, cy) in (y0..=y1).enumerate() {
        let o = (cy - 1) * gw * bpp + (x0 - 1) * bpp;
        plano[o..o + cw].copy_from_slice(&b[k * cw..(k + 1) * cw]);
    }
}

#[cfg(test)]
#[path = "otica_compositor_tests.rs"]
mod tests;
